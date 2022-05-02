// Copyright (c) 2022 The MobileCoin Foundation
extern crate bindgen;

use std::{env, path::PathBuf};

fn main() {
    // Need to evaluate if all the types should be intermixed here
    let bindings = bindgen::Builder::default()
        .header_contents("status.h", "#include <sgx_error.h>\n#include <sgx_urts.h>")
        .clang_arg("-I/opt/intel/sgxsdk/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
