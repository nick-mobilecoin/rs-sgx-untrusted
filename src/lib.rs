// Copyright (c) 2022 The MobileCoin Foundation
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn create_enclave() {
        assert_eq!(sgx_create_enclave(), 4);
    }
}
