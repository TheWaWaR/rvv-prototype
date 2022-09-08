use super::constants::{NP, P2};

extern "C" {
    fn _zz_preload(np: *const u64, p2: *const u64);
    fn _zz_add(r: *mut u64, x: *const u64, y: *const u64, n: u64);
    fn _zz_sub(r: *mut u64, x: *const u64, y: *const u64, n: u64);
    fn _zz_sqr(r: *mut u64, x: *const u64, n: u64);
    fn _zz_neg(r: *mut u64, x: *const u64, n: u64);
    fn _zz_mul(r: *mut u64, x: *const u64, y: *const u64, n: u64);
    fn _zz_mul_scalar(r: *mut u64, x: *const u64, y: *const u64, n: u64);
    fn _zz_add_indexed(
        r: *mut u64,
        x: *const u64,
        y: *const u64,
        x_index: *const u16,
        y_index: *const u16,
        n: u64,
    );
    fn _zz_mul_indexed(
        r: *mut u64,
        x: *const u64,
        y: *const u64,
        x_index: *const u16,
        y_index: *const u16,
        n: u64,
    );
    fn _zz_normalize(r: *mut u64, n: u64);
}

#[inline(always)]
pub fn zz_preload() {
    unsafe { _zz_preload(NP.as_ptr(), P2.as_ptr()) }
}

#[inline(always)]
pub fn zz_add(r: *mut u64, x: *const u64, y: *const u64, n: u64) {
    unsafe { _zz_add(r, x, y, n) }
}

#[inline(always)]
pub fn zz_sub(r: *mut u64, x: *const u64, y: *const u64, n: u64) {
    unsafe { _zz_sub(r, x, y, n) }
}

#[inline(always)]
pub fn zz_sqr(r: *mut u64, x: *const u64, n: u64) {
    unsafe { _zz_sqr(r, x, n) }
}

#[inline(always)]
pub fn zz_neg(r: *mut u64, x: *const u64, n: u64) {
    unsafe { _zz_neg(r, x, n) }
}

#[inline(always)]
pub fn zz_mul(r: *mut u64, x: *const u64, y: *const u64, n: u64) {
    unsafe { _zz_mul(r, x, y, n) }
}

#[inline(always)]
pub fn zz_mul_scalar(r: *mut u64, x: *const u64, y: *const u64, n: u64) {
    unsafe { _zz_mul_scalar(r, x, y, n) }
}

#[inline(always)]
pub fn zz_add_indexed(
    r: *mut u64,
    x: *const u64,
    y: *const u64,
    x_index: *const u16,
    y_index: *const u16,
    n: u64,
) {
    unsafe { _zz_add_indexed(r, x, y, x_index, y_index, n) }
}

#[inline(always)]
pub fn zz_mul_indexed(
    r: *mut u64,
    x: *const u64,
    y: *const u64,
    x_index: *const u16,
    y_index: *const u16,
    n: u64,
) {
    unsafe { _zz_mul_indexed(r, x, y, x_index, y_index, n) }
}

#[inline(always)]
pub fn zz_normalize(r: *mut u64, n: u64) {
    unsafe { _zz_normalize(r, n) }
}
