#![allow(unused_variables)]
#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate rvv;
use ckb_std::default_alloc;

use alloc::format;
use ckb_std::syscalls::debug;

ckb_std::entry!(program_entry);
default_alloc!();

mod rsa_test {
    use rvv_mont_example::rsa::*;
    use rvv_mont_example::uint_version::U256;
    use rvv_mont_example::U;
    pub fn test_rsa() {
        let rsa = Rsa::new();
        let plain_text = U!(2);

        let cipher_text = rsa.encrypt(plain_text);
        // println!("cipher_text = {}", cipher_text);
        let plain_text2 = rsa.decrypt(cipher_text);

        assert_eq!(plain_text, plain_text2);
    }
}

mod uint_version_test {
    use rvv_mont_example::uint_version::*;
    use rvv_mont_example::U;

    pub fn test() {
        let a = U512::from(100);
        let b: U256 = a.into();
        let b2: U512 = b.into();
        assert_eq!(a, b2);
    }

    pub fn test_i512() {
        let a = I512::from(2);
        let b = I512::from(10);
        let sum = a + b;
        assert_eq!(sum, I512::from(12));

        let sub = a - b;
        assert_eq!(sub, I512::new(false, U512::from(8)));

        let mul = a * b;
        assert_eq!(mul, I512::from(20));

        let div = b / a;
        assert_eq!(div, I512::from(5));

        let rem = b % a;
        assert_eq!(rem, I512::from(0));

        assert!(b > a);
        assert!(a < b);
        assert!(a != b);

        let x = I512::new(false, 11.into());
        let y = I512::from(7);
        let rem = x % y;
        assert_eq!(rem, I512::from(3));
    }

    pub fn test_egcd() {
        let a = I512::from(11);
        let b = I512::from(17);
        let (gcd, x, y) = egcd(a, b);
        assert_eq!(gcd, I512::from(1));
        assert_eq!(a * x + b * y, I512::from(1));
    }

    fn test_n(n: U256) {
        let mut mont = Mont::new(n);
        mont.precompute();

        let x = U!(10);
        let x2 = mont.to_mont(x);
        let x3 = mont.reduce(x2.into());
        assert_eq!(x, x3);

        let x = U!(10);
        let y = U!(20);
        // into montgomery form
        let x2 = mont.to_mont(x);
        let y2 = mont.to_mont(y);
        // do multiplication operation
        let xy2 = mont.multi(x2, y2);
        // into normal form
        let xy = mont.reduce(xy2.into());
        // the result should be same
        assert_eq!(xy, (x * y) % mont.n);
    }

    fn test_xy(x: U256, y: U256) {
        let mut mont = Mont::new(U!(1000001));
        mont.precompute();

        // into montgomery form
        let x2 = mont.to_mont(x);
        let y2 = mont.to_mont(y);
        // do multiplication operation
        let xy2 = mont.multi(x2, y2);
        // into normal form
        let xy = mont.reduce(xy2.into());
        // the result should be same
        assert_eq!(xy, (x * y) % mont.n);
    }

    pub fn test_n_loops() {
        for n in 19..100 {
            if n % 2 == 1 {
                test_n(U!(n));
            }
        }
    }

    pub fn test_xy_loops() {
        for x in 10000..10100 {
            test_xy(U!(x), U!(x + 20));
        }
    }

    pub fn test_multiple() {
        let mut mont = Mont::new(U!(17));
        mont.precompute();

        let x = U!(1);
        let y = U!(2);
        let z = U!(3);
        let x2 = mont.to_mont(x);
        let y2 = mont.to_mont(y);
        let z2 = mont.to_mont(z);

        let res = mont.multi(mont.multi(x2, y2), z2);

        let res2 = mont.reduce(res.into());
        assert_eq!(x * y * z % mont.n, res2);
    }

    pub fn test_pow() {
        let mut mont = Mont::new(U!(17));
        mont.precompute();
        for base in 2..5 {
            for n in 5..10 {
                let base = U!(base);
                let n = U!(n);
                let x = mont.to_mont(base);

                let v = mont.pow(x, n);
                let v = mont.reduce(v.into());

                let expected = base.pow(n) % mont.n;
                assert_eq!(expected, v);
            }
        }
    }

    pub fn test_pow2() {
        let mut mont = Mont::new(U!(33));
        mont.precompute();
        let base = U!(2);
        let x = mont.to_mont(base);
        let v = mont.pow(x, U!(7));
        let v = mont.reduce(v.into());
        assert_eq!(v, U!(29));
    }

    pub fn test_pow3() {
        let mut mont = Mont::new(U!(33));
        mont.precompute();
        let x = U!(2);
        let y = U!(2);
        let x2 = mont.to_mont(x);
        let y2 = mont.to_mont(y);
        let z2 = mont.multi(x2, y2);
        let z = mont.reduce(z2.into());
        assert_eq!(z, U!(4));

        let p2 = mont.pow(x2, U!(2));
        let p = mont.reduce(p2.into());
        assert_eq!(p, U!(4));
    }

    pub fn test_ops() {
        let mut mont = Mont::new(U!(33));
        mont.precompute();

        let x = &MontForm::from_u256(U!(2), mont);
        let y = &x.derive_from_u256(U!(3));

        let sum = x + y;
        let res: U256 = sum.into();
        assert_eq!(U!(5), res);

        let sub = y - x;
        assert_eq!(U!(1), sub.into());

        let sub2 = x - y;
        assert_eq!(U!(32), sub2.into());

        let mul = x * y;
        assert_eq!(U!(6), mul.into());

        let pow = x.pow(U!(3));
        assert_eq!(U!(8), pow.into());
    }
}

#[test]
pub fn test() {
    main()
}

pub fn program_entry() -> i8 {
    if cfg!(feature = "use_rvv_vector") {
        debug(format!("feature = use_rvv_vector"));
    } else if cfg!(feature = "use_rvv_asm") {
        debug(format!("feature = use_rvv_asm"));
    } else {
        debug(format!(
            "feature != use_rvv_vector && feature != use_rvv_asm"
        ));
    }
    if cfg!(feature = "simulator") {
        debug(format!("feature = simulator"));
    } else {
        debug(format!("feature != simulator"))
    }

    let mut s = 0;
    for i in 0..10 {
        s += i;
        uint_version_test::test();
        uint_version_test::test_i512();
        uint_version_test::test_egcd();
        uint_version_test::test_n_loops();
        uint_version_test::test_xy_loops();
        uint_version_test::test_multiple();
        uint_version_test::test_pow();
        uint_version_test::test_pow2();
        uint_version_test::test_pow3();
        uint_version_test::test_ops();

        rsa_test::test_rsa();
    }

    debug(format!("done"));
    return s - s;
}
