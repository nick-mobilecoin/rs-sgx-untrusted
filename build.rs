// Copyright (c) 2022 The MobileCoin Foundation
//
// Builds the FFI bindings for the untrusted side of the Intel SGXSDK
extern crate bindgen;
use cargo_emit::{rustc_link_arg, rustc_link_search, warning};
use std::{env, path::PathBuf};

static DEFAULT_SGX_SDK_PATH: &str = "/opt/intel/sgxsdk";

fn sgx_library_path() -> String{
    env::var("SGX_SDK").unwrap_or_else(|_| String::from(DEFAULT_SGX_SDK_PATH))
}

fn sgx_library_suffix() -> String{
    let mode = env::var("SGX_MODE").unwrap_or_else(|_| String::from("SIM"));
    let suffix = match mode.as_str() {
       "SIM" => "_sim",
       "HW" => "",
       mode => {
           warning!("'SGX_MODE' was set to '{}'\n.Should be one of 'SIM' or 'HW', defaulting to 'SIM'", mode);
           "_sim"
       },

    };
    String::from(suffix)
}

fn main() {
    let sim_suffix = sgx_library_suffix();
    rustc_link_arg!(
        &format!("-lsgx_urts{}", sim_suffix),
        &format!("-lsgx_launch{}", sim_suffix)
    );
    rustc_link_search!(&format!("{}/lib64", sgx_library_path()));

    // TODO: This currently brings in *all* of the urts types into one binding.
    //       Need to evaluate if all the types should be intermixed here
    let bindings = bindgen::Builder::default()
        .header_contents("status.h", "#include <sgx_error.h>\n#include <sgx_urts.h>")
        .clang_arg("-I/opt/intel/sgxsdk/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Suppressing warnings from tests, see
        // https://github.com/rust-lang/rust-bindgen/issues/1651
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
