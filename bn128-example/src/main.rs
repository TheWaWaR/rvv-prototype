#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate rvv;
use alt_bn128_example::rvv_impl::bench_mont;

use alloc::format;
use ckb_std::cstr_core::CStr;
use ckb_std::{default_alloc, syscalls::debug};
use core::slice::from_raw_parts;

ckb_std::entry!(program_entry);
default_alloc!();

fn internal_main() {
    // alt_bn128::ethereum::ut::test_alt_bn128_add();
    // debug!("test_alt_bn128_add     pass");
    // alt_bn128::ethereum::ut::test_alt_bn128_mul();
    // debug!("test_alt_bn128_mul     pass");
    if cfg!(feature = "use_rvv_vector") {
        debug(format!("feature = use_rvv_vector"));
    } else {
        debug(format!("feature != use_rvv_vector"));
    }
    if cfg!(feature = "simulator") {
        debug(format!("feature = simulator"));
    } else {
        debug(format!("feature != simulator"))
    }
    if cfg!(feature = "use_rvv_asm") {
        debug(format!("feature = use_rvv_asm"));
    } else {
        debug(format!("feature != use_rvv_asm"));
    }

    if cfg!(feature = "run_all_cases") {
        debug(format!("start test_alt_bn128_pairing ..."));
        alt_bn128_example::ethereum::ut::test_alt_bn128_pairing();
        debug(format!("test_alt_bn128_pairing pass"));
    } else {
        debug(format!("start test_alt_bn128_pairing_once ..."));
        alt_bn128_example::ethereum::ut::test_alt_bn128_pairing_once();
        debug(format!("test_alt_bn128_pairing_once pass"));
    }
}

pub fn program_entry(argc: u64, argv: *const *const u8) -> i8 {
    if argc == 2 {
        let args = unsafe { from_raw_parts(argv, argc as usize) };
        let arg1 = unsafe { CStr::from_ptr(args[1]) };
        let arg1 = arg1.to_str().unwrap();
        if arg1 == "bench_mont" {
            debug(format!("start bench_mont"));
            bench_mont();
            debug(format!("bench_mont done"));
            return 0;
        } else if arg1 == "rvv_crypto_tests" {
            debug(format!("start rvv-crypto tests"));
            alt_bn128_example::rvv_crypto::cross_tests::entry();
            debug(format!("rvv-crypto test done"));
            return 0;
        } else if arg1 == "rvv_crypto_ethereum_tests" {
            debug(format!("start rvv-crypto ethereum tests"));
            alt_bn128_example::rvv_crypto::ethereum_tests::test_entry();
            debug(format!("rvv-crypto ethereum test done"));
            return 0;
        } else if arg1 == "rvv_crypto_ethereum_benches" {
            debug(format!("start rvv-crypto ethereum benches"));
            alt_bn128_example::rvv_crypto::ethereum_tests::bench_entry();
            debug(format!("rvv-crypto ethereum bench done"));
            return 0;
        }
    }

    internal_main();
    return 0;
}
