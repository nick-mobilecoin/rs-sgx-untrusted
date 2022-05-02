// Copyright (c) 2022 The MobileCoin Foundation
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    #[test]
    fn create_enclave() {
        let file_name = CString::new("").unwrap();
        let debug: ::std::os::raw::c_int = 0;
        let mut launch_token: sgx_launch_token_t = [0; 1024];
        let mut launch_token_updated: ::std::os::raw::c_int = 0;
        let mut enclave_id: sgx_enclave_id_t = 0;
        let mut misc_attr: sgx_misc_attribute_t = unsafe{ MaybeUninit::<sgx_misc_attribute_t>::zeroed().assume_init() };
        assert_eq!(unsafe {sgx_create_enclave(file_name.as_ptr(), debug, &mut launch_token as *mut sgx_launch_token_t, &mut launch_token_updated, &mut enclave_id, &mut misc_attr)}, _status_t_SGX_ERROR_ENCLAVE_FILE_ACCESS);
    }
}
