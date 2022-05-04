// Copyright (c) 2022 The MobileCoin Foundation
//
// There is quite a bit going on here.

extern crate bindgen;
use cargo_emit::rerun_if_changed;
use std::env;
use std::path::{Path, PathBuf};
use cc::Build;
use std::process::{Command};

struct EdgerFiles {
    trusted: PathBuf,
    untrusted: PathBuf
}
const DEFAULT_SGX_SDK_PATH: &str = "/opt/intel/sgxsdk";
const EDGER_FILE: &str = "src/enclave.edl";
const ENCLAVE_FILE: &str = "src/enclave.c";
const ENCLAVE_LINKER_SCRIPT: &str = "src/enclave.lds";
const SIGNING_KEY: &str = "src/signing_key.pem";
const ENCLAVE_CONFIG: &str = "src/config.xml";

fn main() {
    let root_dir = root_dir();
    let edger_files = create_enclave_definitions(root_dir.join(EDGER_FILE));

    create_enclave_binary([root_dir.join(ENCLAVE_FILE), edger_files.trusted]);
    create_untrusted_library(&edger_files.untrusted);
    let mut untrusted_header = edger_files.untrusted.clone();
    untrusted_header.set_extension("h");
    create_untrusted_bindings(untrusted_header);
}

fn sgx_library_path() -> String{
    env::var("SGX_SDK").unwrap_or_else(|_| String::from(DEFAULT_SGX_SDK_PATH))
}

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn root_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
}

fn create_enclave_definitions<P: AsRef<Path>>(edl_file: P) -> EdgerFiles {
    rerun_if_changed!(edl_file.as_ref().as_os_str().to_str().expect("Invalid UTF-8 in edl path"));
    let mut command = Command::new(&format!("{}/bin/x64/sgx_edger8r", sgx_library_path()));
    let out_dir = out_dir();
    command.current_dir(&out_dir).arg(edl_file.as_ref().as_os_str());
    let status = command.status().expect("Failed to run edger8r");
    match status.code().unwrap() {
        0 => (),
        _ => panic!("Failed to run edger8")
    }
    let basename = edl_file.as_ref().file_stem().unwrap().to_str().unwrap();

    let trusted = out_dir.join(format!("{}_t.c", basename));
    let untrusted = out_dir.join(format!("{}_u.c", basename));

    EdgerFiles{trusted, untrusted}
}

fn create_enclave_binary<P>(files: P) -> PathBuf
    where
        P: IntoIterator,
        P: Clone,
        P::Item: AsRef<Path>, {
    for file in files.clone() {
        rerun_if_changed!(file.as_ref().as_os_str().to_str().expect("Invalid UTF-8 in enclave c file"));
    }

    Build::new().files(files)
        .include(format!("{}/include", sgx_library_path()))
        .include(format!("{}/include/tlibc", sgx_library_path()))
        .cargo_metadata(false) .shared_flag(true).compile("enclave");

    let static_enclave = out_dir().join("libenclave.a");
    let dynamic_enclave = create_dynamic_enclave_binary(static_enclave);
    sign_enclave_binary(dynamic_enclave)
}

// See https://github.com/alexcrichton/cc-rs/issues/250 for lack of dynamic
// lib in cc crate
fn create_dynamic_enclave_binary<P: AsRef<Path>>(static_enclave: P) -> PathBuf {
    let mut dynamic_enclave = PathBuf::from(static_enclave.as_ref());
    dynamic_enclave.set_extension("so");
    let mut command = Command::new("ld");
    command
        .arg("-o")
        .arg(dynamic_enclave.to_str().expect("Invalid UTF-8 in static enclave path"))
        .args(&["-z", "relro", "-z", "now", "-z", "noexecstack"])
        .arg(&format!("-L{}/lib64/cve_2020_0551_load", sgx_library_path()))
        .arg(&format!("-L{}/lib64", sgx_library_path()))
        .arg("--no-undefined")
        .arg("--nostdlib")
        .arg("--start-group")
        .args(&["--whole-archive", "-lsgx_trts_sim", "--no-whole-archive"])
        .arg(static_enclave.as_ref().to_str().unwrap())
        .args(&["-lsgx_tstdc", "-lsgx_tcxx", "-lsgx_tcrypto", "-lsgx_tservice_sim"])
        .arg("--end-group")
        .arg("-Bstatic")
        .arg("-Bsymbolic")
        .arg("--no-undefined")
        .arg("-pie")
        .arg("-eenclave_entry")
        .arg("--export-dynamic")
        .args(&["--defsym", "__ImageBase=0"])
        .arg("--gc-sections")
        .arg(&format!("--version-script={}", ENCLAVE_LINKER_SCRIPT));

    let status = command.status().expect("Failed to run the linker for dynamic enclave");
    match status.code().unwrap() {
        0 => (),
        _ => panic!("Failed to link the dynamic enclave")
    }
    dynamic_enclave

}

fn sign_enclave_binary<P: AsRef<Path>>(unsigned_enclave: P) -> PathBuf {
    let mut signed_binary = PathBuf::from(unsigned_enclave.as_ref());
    signed_binary.set_extension("signed.so");

    let mut command = Command::new(format!("{}/bin/x64/sgx_sign", sgx_library_path()));
    command.arg("sign") .arg("-enclave") .arg(unsigned_enclave.as_ref())
        .arg("-config") .arg(ENCLAVE_CONFIG)
        .arg("-key") .arg(SIGNING_KEY)
        .arg("-out") .arg(&signed_binary);
    let status = command.status().expect("Failed to execute enclave signer");
    match status.code().unwrap() {
        0 => (),
        _ => panic!("Failed to sign enclave")
    }

    signed_binary
}

fn create_untrusted_library<P: AsRef<Path>>(untrusted_file: P) -> PathBuf {

    Build::new().file(untrusted_file)
        .include(format!("{}/include", sgx_library_path()))
        .include(format!("{}/include/tlibc", sgx_library_path()))
        .compile("untrusted");

    let mut untrusted_object = out_dir();
    untrusted_object.set_file_name("untrusted.a");
    untrusted_object
}

fn create_untrusted_bindings<P: AsRef<Path>>(header: P) {
    let bindings = bindgen::Builder::default()
        .header(header.as_ref().to_str().unwrap())
        .clang_arg(format!("-I{}/include", sgx_library_path()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // See https://github.com/rust-lang/rust-bindgen/issues/1651 for disabling tests
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir().join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
