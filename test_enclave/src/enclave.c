// Copyright (c) 2022 The MobileCoin Foundation
/*
 * A basic functoin that just adds 2 to its input
 */

#include <assert.h>
#include <stdlib.h>
#include "sgx_trts.h"
void ecall_add_2(int input, int *sum) {
//    *sum = input + 2;

    if (sgx_is_within_enclave(sum, sizeof(int)) != 1) {
        abort();
    }
    assert(*sum == 0);
    *sum = input + 2;
}
