extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;

use duckdb::{
    core::{DataChunkHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
    Connection, Result,
};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use std::error::Error;
use libduckdb_sys::{
    duckdb_string_t,
    duckdb_string_t_data,
    duckdb_string_t_length,
};
use duckdb::core::Inserter;
use duckdb::ffi;
use std::{borrow::Cow, slice};
use rust_stemmers::{Algorithm, Stemmer};
struct StandarizeTitles;

mod utils;
use utils::{tame_logic, standard_title_to_bls_title};

fn duckdb_string_to_owned_string(word: &duckdb_string_t) -> String {
    unsafe {
        let len = duckdb_string_t_length(*word);
        let c_ptr = duckdb_string_t_data(word as *const _ as *mut _);
        let bytes = slice::from_raw_parts(c_ptr as *const u8, len as usize);
        String::from_utf8_lossy(bytes).into_owned()
    }
}

fn process_strings(input_slice: &[duckdb_string_t]) -> Vec<String> {
    input_slice
        .iter()
        .map(|word| {
            duckdb_string_to_owned_string(word)
        })
        .collect()
}

impl VScalar for StandarizeTitles {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Extract the input word
        let input_vec = input.flat_vector(0);
        // slice of strings
        let input_slice = input_vec.as_slice_with_len::<duckdb_string_t>(input.len());
        // a flat writable vector
        let output_flat = output.flat_vector();

        let input_strings = process_strings(input_slice);

       let standarized_titles = tame_logic(input_strings.clone());

        for (i, input_string) in input_strings.iter().enumerate() {
            let standard_title = standarized_titles[input_string].as_str();
            let bls_title = standard_title_to_bls_title(standard_title);
            output_flat.insert(i, format!("{} - {}", standard_title, bls_title).as_str());
        }

        Ok(())
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeId::Varchar.into()],
            LogicalTypeId::Varchar.into(),
        )]
    }
}

const FUNCTION_NAME: &str = "standarize_title";

#[duckdb_entrypoint_c_api]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_scalar_function::<StandarizeTitles>(FUNCTION_NAME)
        .expect("Failed to register standarize_title()");
    Ok(())
}