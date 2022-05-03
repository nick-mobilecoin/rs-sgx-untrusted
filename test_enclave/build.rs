// Copyright (c) 2022 The MobileCoin Foundation

use std::env;
use std::path::{Path, PathBuf};
use cc::Build;
use std::process::Command;
use cargo_emit::warning;

struct EdgerFiles {
    trusted: PathBuf,
    untrusted: PathBuf
}
static SGX_TRUSTED_LIBRARY_PATH: &str = "/opt/intel/sgxsdk/lib64";
static EDGER_FILE: &str = "src/enclave.edl";
static ENCLAVE_FILE: &str = "src/enclave.c";

fn main() {
    let edger_files = generate_enclave_definitions(EDGER_FILE);

    generate_enclave_binary([PathBuf::from(ENCLAVE_FILE), edger_files.trusted]);

    generate_untrusted_library(edger_files.untrusted);
}

fn generate_enclave_definitions<P: AsRef<Path>>(edl_file: P) -> EdgerFiles {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    warning!("The output path is {:?}", out_path);
    let mut command = Command::new("/opt/intel/sgxsdk/bin/x64/sgx_edger8r");
    command.current_dir(&out_path).arg(edl_file.as_ref().as_os_str());
    warning!("The command is {:?}", command);
    command.output().expect("Failed to run edger8r");
    let basename = edl_file.as_ref().file_stem().unwrap().to_str().unwrap();

    let mut trusted = out_path.clone();
    trusted.set_file_name(format!("{}_t.c", basename));
    let mut untrusted = out_path.clone();
    untrusted.set_file_name(format!("{}_u.c", basename));

    EdgerFiles{trusted, untrusted}
}

fn generate_enclave_binary<P>(files: P) -> PathBuf
    where
        P: IntoIterator,
        P::Item: AsRef<Path>, {

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut enclave_config = out_path.clone();
    enclave_config.set_file_name("enclave.lds");

    Build::new().files(files)
        .flag("-Wl,--no-undefined")
        .flag("-nostdlib")
        .flag("-nodefaultlibs")
        .flag("-nostartfiles")
        .flag(&format!("-L{}", SGX_TRUSTED_LIBRARY_PATH))
        .flag("-Wl,--whole-archive").flag("-lsgx_trts_sim").flag("-Wl,--no-whole-archive")
        .flag("-lsgx_tstdc").flag("-lsgx_tcxx").flag("-lsgx_crypto_sim").flag("-lsgx_service_sim")
        .flag("-Wl,-Bstatic").flag("-Wl,-Bsymbolic").flag("-Wl,--no-undefined")
        .flag("-Wl,-pie,-eenclave_entry").flag("-Wl,--export-dynamic")
        .flag("-Wl,--defsym,__ImageBase=0").flag("-Wl,--gc-sections")
        .flag(&*format!("-Wl,--version-script={}", enclave_config.to_str().expect("Invalid UTF-8 for OUT_DIR")))
        .shared_flag(true).compile("enclave.so");

    let mut enclave_binary = PathBuf::from(env::var("OUT_DIR").unwrap());
    enclave_binary.set_file_name("enclave.so");
    enclave_binary
}

fn generate_untrusted_library<P: AsRef<Path>>(untrusted_file: P) -> PathBuf {

    Build::new().file(untrusted_file).compile("untrusted");

    let mut untrusted_object = PathBuf::from(env::var("OUT_DIR").unwrap());
    untrusted_object.set_file_name("untrusted.a");
    untrusted_object
}
