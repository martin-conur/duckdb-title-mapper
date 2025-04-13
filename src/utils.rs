use regex::Regex;
use rayon::prelude::*;
use rust_stemmers::{Algorithm, Stemmer};
use sprs::{CsIter, CsMat, CsVec, TriMat};
use ndarray::{Array2, Axis};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use bincode;
use std::fs::File;
use std::io::{Read, Write};
use once_cell::sync::Lazy;

static TOKENIZER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w+").unwrap());
static STEMMER: Lazy<Stemmer> = Lazy::new(|| Stemmer::create(Algorithm::English));

#[derive(Serialize, Deserialize)]
struct TfidfIndex {
    term_to_idx: HashMap<String, usize>,
    doc_freq: HashMap<String, usize>,
    num_docs: usize,
    matrix: CsMat<f64>,
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
    TOKENIZER_RE.find_iter(text)
        .map(|m| m.as_str().to_lowercase())
        .collect()
}

fn stem_tokens(tokens: &[String]) -> Vec<String> {
    tokens.iter()
        .map(|word| STEMMER.stem(word).to_string())
        .collect()
}

fn cosine_similarity_sparse(a: &CsVec<f64>, b: &CsVec<f64>, doc_norms: f64) -> f64 {
    let dot = a.dot(b);
    let norm_a = a.dot(a).sqrt();
    if norm_a > 0.0 && doc_norms > 0.0 {
        dot / (doc_norms * norm_a)
    } else {
        0.0
    }
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
) -> CsMat<f64> {
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
    let query_vecs: Vec<CsVec<f64>> = new_tfidf_matrix.outer_iterator()
    .map(|row| row.to_owned())
    .collect();

    let doc_vecs: Vec<CsVec<f64>> = tfidf_index.matrix.outer_iterator()
        .map(|row| row.to_owned())
        .collect();

    let doc_norms: Vec<f64> =doc_vecs.iter().map(|v| v.dot(v).sqrt()).collect();

     let best_matches = DashMap::new();
    
    query_vecs.into_par_iter().enumerate().for_each(|(i, query_vec)| {
        let mut best_score = -0.0f64;
        let mut best_index = 0;
        for (j, doc_vec) in doc_vecs.iter().enumerate() {
            let score = cosine_similarity_sparse(&query_vec, &doc_vec, doc_norms[j]);
            if score > best_score {
                best_score = score;
                best_index = j;
            }
            
        }

        best_matches.insert(scraped_titles[i].clone(), standard_titles[best_index].clone());
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
