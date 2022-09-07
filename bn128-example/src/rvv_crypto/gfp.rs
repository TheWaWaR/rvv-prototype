use super::zz_rust;
use super::{constants::*, Error};
use crate::arith::U256;
use core::convert::TryFrom;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gfp(pub [u64; 4]);

impl TryFrom<&[u8]> for Gfp {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut e = [0u64; 4];

        for w in 0..4 {
            e[3 - w] = 0;
            for b in 0..8 {
                e[3 - w] += (value[8 * w + b] as u64) << (56 - 8 * b);
            }
        }

        for i in (0..4).rev() {
            if e[i] < P2[i] {
                return Ok(Gfp(e));
            }
            if e[i] > P2[i] {
                return Err("Coordinate exceeds modulus!".into());
            }
        }
        return Err("Coordinate equals modulus!".into());
    }
}

// Gfp::new_from_int64(1)
pub const ONE: Gfp = Gfp([
    15230403791020821917,
    754611498739239741,
    7381016538464732716,
    1011752739694698287,
]);
pub const ZERO: Gfp = Gfp([0, 0, 0, 0]);

impl Gfp {
    pub fn is_zero(&self) -> bool {
        self == &ZERO
    }

    // TODO: do we need a parallel version of exp?
    pub fn exp(&mut self, bits: &[u64; 4]) {
        self.0 = self.exp_to(bits).0;
    }

    pub fn exp_to(&self, bits: &[u64; 4]) -> Self {
        let mut sum = [Gfp(RN1)];
        let mut power = [self.clone()];

        for w in bits {
            for bit in 0..64 {
                if (w >> bit) & 1 == 1 {
                    mul_mov(&mut sum, &power);
                }
                square(&mut power);
            }
        }
        mul_mov_scalar(&mut sum, &Gfp(R3));
        let [a] = sum;
        a
    }

    pub fn invert(&mut self) {
        self.exp(&P_MINUS2)
    }

    pub fn invert_to(&self) -> Self {
        self.exp_to(&P_MINUS2)
    }

    pub fn new_from_int64(x: i64) -> Self {
        let mut arr = if x >= 0 {
            [Gfp([x as u64, 0, 0, 0])]
        } else {
            let mut a = [Gfp([(-x) as u64, 0, 0, 0])];
            neg(&mut a);
            a
        };
        mont_encode(&mut arr);
        let [a] = arr;
        a
    }

    pub fn set(&mut self, a: &Gfp) {
        self.0 = a.0;
    }
}

impl From<Gfp> for U256 {
    fn from(a: Gfp) -> Self {
        let mut arr = [a];
        normalize(&mut arr);
        arr[0].clone().into()
    }
}

// TODO: do we want to introduce transmute to:
// 1. Implement ops for reference types
// 2. Eliminate the clones in assignment ops
impl Add for Gfp {
    type Output = Gfp;

    fn add(self, a: Gfp) -> Gfp {
        let mut arr = [self];
        add_mov(&mut arr[..], &[a]);
        let [r] = arr;
        r
    }
}

impl Mul for Gfp {
    type Output = Gfp;

    fn mul(self, a: Gfp) -> Gfp {
        let mut arr = [self];
        mul_mov(&mut arr[..], &[a]);
        let [r] = arr;
        r
    }
}

impl Neg for Gfp {
    type Output = Gfp;

    fn neg(self) -> Gfp {
        let mut arr = [self];
        neg(&mut arr[..]);
        let [r] = arr;
        r
    }
}

impl Sub for Gfp {
    type Output = Gfp;

    fn sub(self, a: Gfp) -> Gfp {
        let mut arr = [self];
        sub_mov(&mut arr[..], &[a]);
        let [r] = arr;
        r
    }
}

impl AddAssign for Gfp {
    fn add_assign(&mut self, other: Gfp) {
        let mut arr = [self.clone()];
        add_mov(&mut arr[..], &[other]);
        self.0 = arr[0].0;
    }
}

impl SubAssign for Gfp {
    fn sub_assign(&mut self, other: Gfp) {
        let mut arr = [self.clone()];
        sub_mov(&mut arr[..], &[other]);
        self.0 = arr[0].0;
    }
}

pub fn double(dst: &mut [Gfp]) {
    do_double(dst.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn double_to(src: &[Gfp], dst: &mut [Gfp]) {
    debug_assert_eq!(src.len(), dst.len());
    do_double(src.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn mul(a: &[Gfp], b: &[Gfp], c: &mut [Gfp]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(b.len(), c.len());

    do_mul(a.as_ptr(), b.as_ptr(), c.as_mut_ptr(), c.len());
}

pub fn mul_mov(dst: &mut [Gfp], src: &[Gfp]) {
    debug_assert_eq!(dst.len(), src.len());

    do_mul(dst.as_ptr(), src.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn square(dst: &mut [Gfp]) {
    do_square(dst.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn square_to(src: &[Gfp], dst: &mut [Gfp]) {
    debug_assert_eq!(src.len(), dst.len());

    do_square(src.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn mul_mov_scalar(dst: &mut [Gfp], src: &Gfp) {
    do_mul_scalar(dst.as_ptr(), src, dst.as_mut_ptr(), dst.len());
}

pub fn mul_scalar(a: &[Gfp], b: &Gfp, c: &mut [Gfp]) {
    debug_assert_eq!(a.len(), c.len());

    do_mul_scalar(a.as_ptr(), b, c.as_mut_ptr(), c.len());
}

pub fn add_mov(dst: &mut [Gfp], src: &[Gfp]) {
    debug_assert_eq!(dst.len(), src.len());

    do_add(dst.as_ptr(), src.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn add(a: &[Gfp], b: &[Gfp], c: &mut [Gfp]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(b.len(), c.len());

    do_add(a.as_ptr(), b.as_ptr(), c.as_mut_ptr(), c.len());
}

pub fn sub_mov(dst: &mut [Gfp], src: &[Gfp]) {
    debug_assert_eq!(dst.len(), src.len());

    do_sub(dst.as_ptr(), src.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn sub(a: &[Gfp], b: &[Gfp], c: &mut [Gfp]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(b.len(), c.len());

    do_sub(a.as_ptr(), b.as_ptr(), c.as_mut_ptr(), c.len());
}

pub fn neg(dst: &mut [Gfp]) {
    do_neg(dst.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn neg_to(src: &[Gfp], dst: &mut [Gfp]) {
    debug_assert_eq!(dst.len(), src.len());

    do_neg(src.as_ptr(), dst.as_mut_ptr(), dst.len());
}

pub fn mont_encode(dst: &mut [Gfp]) {
    mul_mov_scalar(dst, &Gfp(R2));
}

pub fn mont_decode(dst: &mut [Gfp]) {
    mul_mov_scalar(dst, &Gfp([1, 0, 0, 0]));
}

#[inline(always)]
pub fn do_square(src: *const Gfp, dst: *mut Gfp, len: usize) {
    zz_rust::zz_sqr(dst as *mut u64, src as *const u64, len as u64);
}

#[inline(always)]
pub fn do_double(src: *const Gfp, dst: *mut Gfp, len: usize) {
    zz_rust::zz_add(
        dst as *mut u64,
        src as *const u64,
        src as *const u64,
        len as u64,
    );
}

#[inline(always)]
pub fn do_neg(src: *const Gfp, dst: *mut Gfp, len: usize) {
    zz_rust::zz_neg(dst as *mut u64, src as *const u64, len as u64)
}

/// Some input values might be larger than p, this normalizes the value so they
/// remain regular
#[inline(always)]
pub fn normalize(dst: &mut [Gfp]) {
    zz_rust::zz_normalize(dst.as_mut_ptr() as *mut u64, dst.len() as u64);
}

#[inline(always)]
pub fn do_mul(a: *const Gfp, b: *const Gfp, c: *mut Gfp, len: usize) {
    zz_rust::zz_mul(c as *mut u64, a as *const u64, b as *const u64, len as u64);
}

#[inline(always)]
pub fn do_mul_scalar(a: *const Gfp, b: &Gfp, c: *mut Gfp, len: usize) {
    zz_rust::zz_mul_scalar(
        c as *mut u64,
        a as *const u64,
        b as *const Gfp as *const u64,
        len as u64,
    );
}

#[inline(always)]
pub fn add_by_byte_index(a: &[Gfp], b: &[Gfp], a_index: &[u16], b_index: &[u16], c: &mut [Gfp]) {
    zz_rust::zz_add_indexed(
        c.as_mut_ptr() as *mut u64,
        a.as_ptr() as *const u64,
        b.as_ptr() as *const u64,
        a_index.as_ptr() as *const u16,
        b_index.as_ptr() as *const u16,
        a_index.len() as u64,
    );
}

#[inline(always)]
pub fn mul_by_byte_index(a: &[Gfp], b: &[Gfp], a_index: &[u16], b_index: &[u16], c: &mut [Gfp]) {
    zz_rust::zz_mul_indexed(
        c.as_mut_ptr() as *mut u64,
        a.as_ptr() as *const u64,
        b.as_ptr() as *const u64,
        a_index.as_ptr() as *const u16,
        b_index.as_ptr() as *const u16,
        a_index.len() as u64,
    );
}

#[inline(always)]
pub fn do_add(a: *const Gfp, b: *const Gfp, c: *mut Gfp, len: usize) {
    zz_rust::zz_add(c as *mut u64, a as *const u64, b as *const u64, len as u64);
}

#[inline(always)]
pub fn do_sub(a: *const Gfp, b: *const Gfp, c: *mut Gfp, len: usize) {
    zz_rust::zz_sub(c as *mut u64, a as *const u64, b as *const u64, len as u64);
}
