// Copyright (c) 2022 The MobileCoin Foundation

use std::env;
use std::path::{Path, PathBuf};
use cc::Build;
use std::process::Command;

struct EdgerFiles {
    trusted: PathBuf,
    untrusted: PathBuf
}
static SGX_TRUSTED_LIBRARY_PATH: &str = "/opt/intel/sgxsdk/lib64";
static EDGER_FILE: &str = "src/enclave.edl";
static ENCLAVE_FILE: &str = "src/enclave.c";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut enclave_config = out_path.clone();
    enclave_config.set_file_name("enclave.lds");
    let edger_files = generate_enclave_definitions(EDGER_FILE);

    Build::new().files([ENCLAVE_FILE, edger_files.trusted])
        .flag("-Wl,--no-undefined")
        .flag("-nostdlib")
        .flag("-nodefaultlibs")
        .flag("-nostartfiles")
        .flag("-LSGX_TRUSTED_LIBRARY_PATH")
        .flag("-Wl,--whole-archive").flag("-lsgx_trts_sim").flag("-Wl,--no-whole-archive")
        .flag("-lsgx_tstdc").flag("-lsgx_tcxx").flag("-lsgx_crypto_sim").flag("-lsgx_service_sim")
        .flag("-Wl,-Bstatic").flag("-Wl,-Bsymbolic").flag("-Wl,--no-undefined")
        .flag("-Wl,-pie,-eenclave_entry").flag("-Wl,--export-dynamic")
        .flag("-Wl,--defsym,__ImageBase=0").flag("-Wl,--gc-sections")
        .flag(&*format!("-Wl,--version-script={}", enclave_config.to_str().expect("Invalid UTF-8 for OUT_DIR")))
        .shared_flag(true).compile("enclave.so");

}

fn generate_enclave_definitions<P: AsRef<Path>>(edl_file: P) -> EdgerFiles {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let command = Command::new("/opt/intel/sgxsdk/bin/x64/sgx_edger8r").current_dir(out_path).arg(&edl_file);
    command.output().expect("Failed to run edger8r");
    let basename = edl_file.file_prefix().unwrap();
    let trusted = PathBuf::from(&edl_file).set_file_name(format!("{}_t.c", basename)).unwrap();
    let untrusted = PathBuf::from(&edl_file).set_file_name(format!("{}_u.c", basename)).unwrap();
    EdgerFiles{trusted, untrusted}
}