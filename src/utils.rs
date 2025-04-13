use regex::Regex;
use rayon::prelude::*;
use rust_stemmers::{Algorithm, Stemmer};
use sprs::TriMat;
use ndarray::{Array2, Axis};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use bincode;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize)]
struct TfidfIndex {
    term_to_idx: HashMap<String, usize>,
    doc_freq: HashMap<String, usize>,
    num_docs: usize,
    matrix: Vec<Vec<f64>>,
}

pub fn load_standarized_values() -> Vec<String> {
    let json_data = include_str!("../resources/standarized_titles.json");
    let titles_list: Vec<Value> = serde_json::from_str(json_data).expect("JSON was not well-formatted");

    titles_list.iter()
        .filter_map(|entry| entry.get("other_titles"))
        .flat_map(|titles| titles.as_array())
        .flatten()
        .filter_map(|title| title.as_str().map(String::from))
        .collect()
}

pub fn get_standardized_titles() -> Vec<String> {
    load_standarized_values()
}

fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r"\w+").unwrap();
    re.find_iter(text)
        .map(|m| m.as_str().to_lowercase())
        .collect()
}

fn stem_tokens(tokens: &[String]) -> Vec<String> {
    let stemmer = Stemmer::create(Algorithm::English);
    tokens.iter()
        .map(|word| stemmer.stem(word).to_string())
        .collect()
}

fn build_tfidf_index(docs: &[String]) -> TfidfIndex {
    let tokenized: Vec<Vec<String>> = docs.par_iter().map(|doc| tokenize(doc)).collect();
    let stemmed: Vec<Vec<String>> = tokenized.par_iter().map(|t| stem_tokens(t)).collect();

    let mut term_to_idx: HashMap<String, usize> = HashMap::new();
    let mut doc_freq: HashMap<String, usize> = HashMap::new();

    for doc in &stemmed {
        let mut unique_terms = HashSet::new();
        for term in doc {
            unique_terms.insert(term.clone());
            let next_index = term_to_idx.len();
            term_to_idx.entry(term.clone()).or_insert(next_index);
        }
        for term in unique_terms {
            *doc_freq.entry(term).or_insert(0) += 1;
        }
    }

    let matrix = compute_tfidf_matrix(&stemmed, &term_to_idx, &doc_freq, docs.len());
    TfidfIndex { term_to_idx, doc_freq, num_docs: docs.len(), matrix }
}

fn compute_tfidf_matrix(
    stemmed_docs: &[Vec<String>],
    term_to_idx: &HashMap<String, usize>,
    doc_freq: &HashMap<String, usize>,
    num_docs: usize,
) -> Vec<Vec<f64>> {
    let num_terms = term_to_idx.len();
    let matrix_mutex = Mutex::new(TriMat::new((stemmed_docs.len(), num_terms)));

    stemmed_docs.par_iter().enumerate().for_each(|(doc_idx, doc)| {
        let mut term_counts: HashMap<&String, usize> = HashMap::new();
        for term in doc {
            *term_counts.entry(term).or_insert(0) += 1;
        }

        let mut local_triplets = Vec::new();
        for (term, count) in term_counts {
            if let Some(&term_idx) = term_to_idx.get(term) {
                let tf = count as f64 / doc.len() as f64;
                let idf = (num_docs as f64 / doc_freq.get(term).copied().unwrap_or(1) as f64).ln();
                local_triplets.push((doc_idx, term_idx, tf * idf));
            }
        }

        let mut matrix = matrix_mutex.lock().unwrap();
        for (r, c, v) in local_triplets {
            matrix.add_triplet(r, c, v);
        }
    });

    let csr_matrix = matrix_mutex.into_inner().unwrap().to_csr::<usize>();

    csr_matrix
        .outer_iterator()
        .map(|row| {
            let mut vec = vec![0.0; num_terms];
            for (idx, &value) in row.iter() {
                vec[idx] = value;
            }
            vec
        })
        .collect()
}

fn cosine_similarity(vec1: Vec<Vec<f64>>, vec2: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mat1 = Array2::from_shape_vec((vec1.len(), vec1[0].len()), vec1.into_iter().flatten().collect()).unwrap();
    let mat2 = Array2::from_shape_vec((vec2.len(), vec2[0].len()), vec2.into_iter().flatten().collect()).unwrap();

    let norm_mat1: Vec<f64> = mat1.axis_iter(Axis(0)).map(|row| row.mapv(|x| x.powi(2)).sum().sqrt()).collect();
    let norm_mat2: Vec<f64> = mat2.axis_iter(Axis(0)).map(|row| row.mapv(|x| x.powi(2)).sum().sqrt()).collect();

    let similarity = mat1.dot(&mat2.t());

    similarity.outer_iter().enumerate().map(|(i, row)| {
        row.iter().enumerate().map(|(j, &val)| {
            let norm_product = norm_mat1[i] * norm_mat2[j];
            if norm_product > 0.0 {
                val / norm_product
            } else {
                0.0
            }
        }).collect()
    }).collect()
}

pub fn tame_logic(scraped_titles: Vec<String>) -> HashMap<String, String> {
    let standard_titles = get_standardized_titles();

    let tfidf_index = if let Ok(mut file) = File::open("precomputed_tfidf_index.bin") {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read TF-IDF index");
        bincode::deserialize::<TfidfIndex>(&buffer).expect("Failed to deserialize TF-IDF index")
    } else {
        let index = build_tfidf_index(&standard_titles);
        let encoded = bincode::serialize(&index).expect("Failed to serialize TF-IDF index");
        let mut file = File::create("precomputed_tfidf_index.bin").expect("Failed to create TF-IDF index file");
        file.write_all(&encoded).expect("Failed to write TF-IDF index file");
        index
    };

    let tokenized: Vec<Vec<String>> = scraped_titles.par_iter().map(|doc| tokenize(doc)).collect();
    let stemmed: Vec<Vec<String>> = tokenized.par_iter().map(|t| stem_tokens(t)).collect();

    let new_tfidf_matrix = compute_tfidf_matrix(
        &stemmed,
        &tfidf_index.term_to_idx,
        &tfidf_index.doc_freq,
        tfidf_index.num_docs,
    );

    let sim_vecs = cosine_similarity(new_tfidf_matrix, tfidf_index.matrix);

    let best_matches = DashMap::new();
    scraped_titles.par_iter().enumerate().for_each(|(i, doc1)| {
        if let Some((best_idx, _)) = sim_vecs[i]
            .iter()
            .enumerate()
            .max_by(|(_, sim1), (_, sim2)| sim1.partial_cmp(sim2).unwrap_or(std::cmp::Ordering::Equal))
        {
            best_matches.insert(doc1.clone(), standard_titles[best_idx].clone());
        }
    });

    best_matches.into_iter().collect()
}

pub fn standard_title_to_bls_title(standard_title: &str) -> String {
    let json_data = include_str!("../resources/standarized_titles.json");
    let titles_list: Vec<Value> = serde_json::from_str(json_data).expect("JSON was not well-formatted");

    let bls_titles: HashMap<String, String> = titles_list
        .iter()
        .filter_map(|entry| {
            let title_name = entry.get("title_name")?.as_str()?.to_string();
            let mut titles = entry
                .get("other_titles")
                .and_then(|titles| titles.as_array())
                .map(|titles| {
                    titles
                        .iter()
                        .filter_map(|title| title.as_str().map(String::from))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();
            titles.push(title_name.clone());
            Some((title_name, titles))
        })
        .flat_map(|(title_name, titles)| titles.into_iter().map(move |title| (title, title_name.clone())))
        .collect();

    let bls_title_map: HashMap<String, String> = bls_titles.into_iter().collect();

    bls_title_map
        .get(standard_title)
        .unwrap_or(&standard_title.to_string())
        .to_string()
}
