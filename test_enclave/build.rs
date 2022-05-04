// Copyright (c) 2022 The MobileCoin Foundation

use std::env;
use std::path::{Path, PathBuf};
use cc::Build;
use std::process::{Command};

struct EdgerFiles {
    trusted: PathBuf,
    untrusted: PathBuf
}
static SGX_TRUSTED_LIBRARY_PATH: &str = "/opt/intel/sgxsdk/lib64";
static EDGER_FILE: &str = "src/enclave.edl";
static ENCLAVE_FILE: &str = "src/enclave.c";
const CURRENT_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "..", "/", file!());

fn main() {
    let current_file = PathBuf::from(CURRENT_FILE);
    let root_path = current_file.parent().unwrap();
    let edger_files = create_enclave_definitions(PathBuf::from(&root_path).join(EDGER_FILE));

    create_enclave_binary([PathBuf::from(root_path).join(ENCLAVE_FILE), edger_files.trusted]);

    create_untrusted_library(edger_files.untrusted);
}

fn create_enclave_definitions<P: AsRef<Path>>(edl_file: P) -> EdgerFiles {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut command = Command::new("/opt/intel/sgxsdk/bin/x64/sgx_edger8r");
    command.current_dir(&out_path).arg(edl_file.as_ref().as_os_str());
    let status = command.status().expect("Failed to run edger8r");
    match status.code().unwrap() {
        0 => (),
        _ => panic!("Failed to run edger8")
    }
    let basename = edl_file.as_ref().file_stem().unwrap().to_str().unwrap();

    let trusted = out_path.join(format!("{}_t.c", basename));
    let untrusted = out_path.join(format!("{}_u.c", basename));

    EdgerFiles{trusted, untrusted}
}

fn create_enclave_binary<P>(files: P) -> PathBuf
    where
        P: IntoIterator,
        P::Item: AsRef<Path>, {

    Build::new().files(files)
        .include("/opt/intel/sgxsdk/include")
        .include("/opt/intel/sgxsdk/include/tlibc")
        .shared_flag(true).compile("enclave");

    let static_enclave = PathBuf::from(env::var("OUT_DIR").unwrap()).join("libenclave.a");
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
        .arg(&format!("-L{}", SGX_TRUSTED_LIBRARY_PATH))
        .arg("--no-undefined")
        .arg("--nostdlib")
        .args(&["--whole-archive", "-lsgx_trts_sim", "--no-whole-archive"])
        .arg(static_enclave.as_ref().to_str().unwrap())
        .args(&["-lsgx_tstdc", "-lsgx_tcxx", "-lsgx_tcrypto", "-lsgx_tservice_sim"])
        .arg("-Bstatic")
        .arg("-Bsymbolic")
        .arg("--no-undefined")
        .arg("-pie")
        .arg("-eenclave_entry")
        .arg("--export-dynamic")
        .args(&["--defsym", "__ImageBase=0"])
        .arg("--gc-sections")
        .arg("--version-script=enclave.lds");

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

    let mut command = Command::new("/opt/intel/sgxsdk/bin/x64/sgx_sign");
    command.arg("sign") .arg("-enclave") .arg(unsigned_enclave.as_ref())
        .arg("-config") .arg("config.xml")
        .arg("-key") .arg("signing_key.pem")
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
        .include("/opt/intel/sgxsdk/include")
        .include("/opt/intel/sgxsdk/include/tlibc")
        .compile("untrusted");

    let mut untrusted_object = PathBuf::from(env::var("OUT_DIR").unwrap());
    untrusted_object.set_file_name("untrusted.a");
    untrusted_object
}
