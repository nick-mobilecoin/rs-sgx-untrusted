// Copyright (c) 2022 The MobileCoin Foundation
// See https://download.01.org/intel-sgx/sgx-dcap/1.9/linux/docs/Intel_SGX_Enclave_Common_Loader_API_Reference.pdf
//
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;
use std::mem::MaybeUninit;
use std::os::raw::c_int;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Default)]
pub struct Enclave {
    // The filename of the enclave
    filename: CString,

    // The enclave ID, assigned by the sgx interface
    // Will be None when the enclave has not been created.
    id: Option<sgx_enclave_id_t>,

    // True if the enclave should be created in debug mode
    debug: bool,
}

impl Enclave {
    fn new(filename: &str) -> Enclave {
        let filename = CString::new(filename).expect("Can't convert enclave filename to CString.");
        Enclave{filename, ..Default::default()}
    }

    fn create(mut self) -> sgx_status_t {
        let mut launch_token: sgx_launch_token_t = [0; 1024];
        let mut launch_token_updated: c_int = 0;
        let mut misc_attr: sgx_misc_attribute_t = unsafe{ MaybeUninit::<sgx_misc_attribute_t>::zeroed().assume_init() };
        let mut enclave_id: sgx_enclave_id_t = 0;
        let result = unsafe {sgx_create_enclave(self.filename.as_ptr(), self.debug as c_int, &mut launch_token as *mut sgx_launch_token_t, &mut launch_token_updated, &mut enclave_id, &mut misc_attr)};
        if result == _status_t_SGX_SUCCESS {
            self.id = Some(enclave_id);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_to_create_enclave_with_non_existent_file() {
        let enclave = Enclave::new("does_not_exist.signed.so");
        // assert_eq!(enclave.create(), _status_t_SGX_SUCCESS);
        assert_eq!(enclave.create(), _status_t_SGX_ERROR_ENCLAVE_FILE_ACCESS);
    }
}
