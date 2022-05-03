// Copyright (c) 2022 The MobileCoin Foundation

use std::env;
use std::path::{Path, PathBuf};
use cc::Build;
use std::process::{Command};
use cargo_emit::warning;

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

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut version_script = out_path.clone();
    version_script.set_file_name("enclave.lds");

    Build::new().files(files)
        .include("/opt/intel/sgxsdk/include")
        .include("/opt/intel/sgxsdk/include/tlibc")
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
        .flag(&*format!("-Wl,--version-script={}", version_script.to_str().expect("Invalid UTF-8 for OUT_DIR")))
        .shared_flag(true).compile("enclave.so");

    let mut enclave_binary = PathBuf::from(env::var("OUT_DIR").unwrap());
    enclave_binary.set_file_name("enclave.so");
    sign_enclave_binary(enclave_binary)
}

fn sign_enclave_binary<P: AsRef<Path>>(unsigned_enclave: P) -> PathBuf {
    let mut signed_binary = PathBuf::from(unsigned_enclave.as_ref());
    signed_binary.set_extension("signed.so");

    let mut command = Command::new("/opt/intel/sgxsdk/bin/x64/sgx_sign");
    command.arg("sign") .arg("-enclave") .arg(unsigned_enclave.as_ref())
        .arg("-config") .arg("")
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
