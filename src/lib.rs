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
    // Will be `None` when the enclave has not been created.
    id: Option<sgx_enclave_id_t>,

    // `true` if the enclave should be created in debug mode
    debug: bool,
}

impl Enclave {
    pub fn new(filename: &str) -> Enclave {
        let filename = CString::new(filename).expect("Can't convert enclave filename to CString.");
        Enclave {
            filename,
            ..Default::default()
        }
    }

    pub fn debug(&mut self, debug: bool) -> &mut Enclave {
        self.debug = debug;
        self
    }

    pub fn create(&mut self) -> sgx_status_t {
        let mut launch_token: sgx_launch_token_t = [0; 1024];
        let mut launch_token_updated: c_int = 0;
        let mut misc_attr: sgx_misc_attribute_t =
            unsafe { MaybeUninit::<sgx_misc_attribute_t>::zeroed().assume_init() };
        let mut enclave_id: sgx_enclave_id_t = 0;
        let result = unsafe {
            sgx_create_enclave(
                self.filename.as_ptr(),
                self.debug as c_int,
                &mut launch_token as *mut sgx_launch_token_t,
                &mut launch_token_updated,
                &mut enclave_id,
                &mut misc_attr,
            )
        };
        if result == _status_t_SGX_SUCCESS {
            self.id = Some(enclave_id);
        }
        result
    }
}
impl Drop for Enclave {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            // Per the docs, this will only return SGX_SUCCESS or
            // SGX_ERROR_INVALID_ENCLAVE_ID the invalid ID error will only
            // happen when the id is invalid, the enclave hasn't been loaded,
            // or the enclave has already been destroyed. Any of these cases
            // don't afford corrective action, so ignore the return value
            unsafe { sgx_destroy_enclave(id) };
            self.id = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_enclave::ENCLAVE;

    #[test]
    fn fail_to_create_enclave_with_non_existent_file() {
        let mut enclave = Enclave::new("does_not_exist.signed.so");
        assert_eq!(enclave.create(), _status_t_SGX_ERROR_ENCLAVE_FILE_ACCESS);
    }

    #[test]
    fn create_enclave_with_existent_file() {
        let mut enclave = Enclave::new(ENCLAVE);
        assert_eq!(enclave.create(), _status_t_SGX_SUCCESS);
    }

    #[test]
    fn test_default_debug_flag_is_0() {
        // For the debug flag it's not easy, in a unit test, to test it was
        // passed to `sgx_create_enclave()`, instead we focus on the
        // `as c_int` portion maps correctly to 0 or 1
        let enclave = Enclave::new("");
        assert_eq!(enclave.debug as c_int, 0);
    }

    #[test]
    fn test_when_debug_flag_is_true_it_is_1() {
        let mut enclave = Enclave::new("");
        enclave.debug(true);
        assert_eq!(enclave.debug as c_int, 1);
    }
}
