use super::{constants::*, Error};
use crate::arith::U256;
use core::convert::TryFrom;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
use rvv_asm::rvv_asm;

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

/// WARNING: all functions involving inline rvv assembly must explicitly be
/// marked with `#[inlnie(never)]`!!!!!!!
/// We have noticed errors when multiple functions here are called together,
/// inlining them would lead to the compiler optimizing away certain operations.
/// A more proper way should be adding memory barriers, until we can find the
/// correct way for inserting memory barriers, we have to mark then as non-inlinable.
#[inline(never)]
pub fn do_square(src: *const Gfp, dst: *mut Gfp, len: usize) {
    unsafe {
        // 4 registers as a group, that gives us 8 free v registers to use
        // t1: vl
        // t2: remaining element length
        // t3: source address variable
        // t4: destination address variable
        // t5: free variable
        // Only v0, v4, v8, v16, v24 and v28 are used. v8/v16 can be used as
        // 8-register group, rest are only used as 4-register group
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {src}",
            "mv t4, {dst}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operands
            "vle256.v v0, (t3)",
            "vle256.v v2, (t3)",
            // T = mul(a, b) => v8
            "vwmulu.vv v4, v0, v2",
            // Extract T[0..4] => v0
            "vnsrl.wx v0, v4, x0",
            // m = halfMul(T[0..4], np) => v4
            "vmul.vv v2, v0, v28",
            // t = mul(m, p2)=> v16
            "vwmulu.vv v8, v2, v30",
            // c = t + T = > v8, with carry in v0
            // Temporarily enlarging vlen to deal with bigger adds
            "vsetvli t1, t1, e512, m2",
            "vmadc.vv v0, v4, v8",
            "vadd.vv v4, v4, v8",
            "vsetvli t1, t1, e256, m2",
            // Extract c[4..8] => v4
            "li t5, 256",
            "vnsrl.wx v2, v4, t5",
            // gfpCarry using v4 in c[4..8], with carry in v0
            // c[4..8] - p2 => v16, with carry in v8
            "vmsbc.vv v4, v2, v30",
            "vsub.vv v8, v2, v30",
            // Combine carries
            "vmandnot.mm v0, v4, v0",
            // Select value, if carry is 1, use value in v4, otherwise use value in v16
            "vmerge.vvm v2, v8, v2, v0",
            // Store result
            "vse256.v v2, (t4)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            // t3/t4, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t3, t3, t1",
            "add t4, t4, t1",
            "blt x0, t2, 1b",
            len = in (reg) len,
            src = in (reg) src,
            dst = in (reg) dst,
        );
    }
}

#[inline(never)]
pub fn do_double(src: *const Gfp, dst: *mut Gfp, len: usize) {
    unsafe {
        // 8 registers as a group since add is simple and can do with less
        // registers
        // t1: vl
        // t2: remaining element length
        // t3: source address variable
        // t4: destination address variable
        // t5: free variable
        // v0, v8, v16, v24 are used.
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {src}",
            "mv t4, {dst}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operands
            "vle256.v v2, (t3)",
            "vle256.v v4, (t3)",
            // Add operands together
            // c = a + b => v8, with carry in v0
            "vmadc.vv v0, v2, v4",
            "vadd.vv v2, v2, v4",
            // gfpCarry on c
            // c - p2 => v24, with carry in v16
            "vmsbc.vv v4, v2, v30",
            "vsub.vv v6, v2, v30",
            // Combine carries
            "vmandnot.mm v0, v4, v0",
            // Select value, if carry is 1, use value in v8 (c),
            // otherwise use value in v24 (c - p2)
            "vmerge.vvm v2, v6, v2, v0",
            // Store result
            "vse256.v v2, (t4)",
            // Update t2/t3, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            // t3/t4, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t3, t3, t1",
            "add t4, t4, t1",
            "blt x0, t2, 1b",
            len = in (reg) len,
            dst = in (reg) dst,
            src = in (reg) src,
        );
    }
}

#[inline(never)]
pub fn do_neg(src: *const Gfp, dst: *mut Gfp, len: usize) {
    unsafe {
        // 8 registers as a group since add is simple and can do with less
        // registers
        // t1: vl
        // t2: remaining element length
        // t3: source address variable
        // t4: destination address variable
        // t5: free variable
        // v0, v8, v16, v24 are used.
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {src}",
            "mv t4, {dst}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operand d into v8
            "vle256.v v2, (t3)",
            // c = p2 - d => v16, carry is cleared in v0
            "vsub.vv v4, v30, v2",
            "vmxor.mm v0, v0, v0",
            // gfpCarry on c with carry
            // c - p2 => v24, with carry in v8
            "vmsbc.vv v2, v4, v30",
            "vsub.vv v6, v4, v30",
            // Combine carries
            "vmandnot.mm v0, v2, v0",
            // Select value, if carry is 1, use value in v16 (c),
            // otherwise use value in v24 (c - p2)
            "vmerge.vvm v2, v6, v4, v0",
            // Store result
            "vse256.v v2, (t4)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            // t3/t4, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t3, t3, t1",
            "add t4, t4, t1",
            "blt x0, t2, 1b",
            len = in (reg) len,
            dst = in (reg) dst,
            src = in (reg) src,
        );
    }
}

/// Some input values might be larger than p, this normalizes the value so they
/// remain regular
#[inline(never)]
pub fn normalize(dst: &mut [Gfp]) {
    unsafe {
        // 8 registers as a group since add is simple and can do with less
        // registers
        // t1: vl
        // t2: remaining element length
        // t3: destination/source address variables
        // t5: free variable
        // v0, v8, v16, v24 are used.
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {dst}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operand c
            "vle256.v v2, (t3)",
            // Clear mask
            "vmxor.mm v0, v0, v0",
            // gfpCarry
            // c - p2 => v24, with carry in v16
            "vmsbc.vv v4, v2, v30",
            "vsub.vv v6, v2, v30",
            // Combine carries
            "vmandnot.mm v0, v4, v0",
            // Select value, if carry is 1, use value in v8 (c),
            // otherwise use value in v24 (c - p2)
            "vmerge.vvm v2, v6, v2, v0",
            // Store result
            "vse256.v v2, (t3)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            // t3/t4, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t3, t3, t1",
            "blt x0, t2, 1b",
            len = in (reg) dst.len(),
            dst = in (reg) dst.as_ptr(),
        );
    }
}

#[inline(never)]
pub fn do_mul(a: *const Gfp, b: *const Gfp, c: *mut Gfp, len: usize) {
    unsafe {
        // 4 registers as a group, that gives us 8 free v registers to use
        // t1: vl
        // t2: remaining element length
        // t3/t4: source address variables
        // t5: free variable
        // t6: destination variable
        // Only v0, v4, v8, v16, v24 and v28 are used. v8/v16 can be used as
        // 8-register group, rest are only used as 4-register group
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {a}",
            "mv t4, {b}",
            "mv t6, {c}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operands
            "vle256.v v0, (t3)",
            "vle256.v v2, (t4)",
            // T = mul(a, b) => v8
            "vwmulu.vv v4, v0, v2",
            // Extract T[0..4] => v0
            "vnsrl.wx v0, v4, x0",
            // m = halfMul(T[0..4], np) => v4
            "vmul.vv v2, v0, v28",
            // t = mul(m, p2)=> v16
            "vwmulu.vv v8, v2, v30",
            // c = t + T = > v8, with carry in v0
            // Temporarily enlarging vlen to deal with bigger adds
            "vsetvli t1, t1, e512, m2",
            "vmadc.vv v0, v4, v8",
            "vadd.vv v4, v4, v8",
            "vsetvli t1, t1, e256, m2",
            // Extract c[4..8] => v4
            "li t5, 256",
            "vnsrl.wx v2, v4, t5",
            // gfpCarry using v4 in c[4..8], with carry in v0
            // c[4..8] - p2 => v16, with carry in v8
            "vmsbc.vv v4, v2, v30",
            "vsub.vv v8, v2, v30",
            // Combine carries
            "vmandnot.mm v0, v4, v0",
            // Select value, if carry is 1, use value in v4, otherwise use value in v16
            "vmerge.vvm v2, v8, v2, v0",
            // Store result
            "vse256.v v2, (t6)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            // t3/t4, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t3, t3, t1",
            "add t4, t4, t1",
            "add t6, t6, t1",
            "blt x0, t2, 1b",
            len = in (reg) len,
            a = in (reg) a,
            b = in (reg) b,
            c = in (reg) c,
        );
    }
}

#[inline(never)]
pub fn do_mul_scalar(a: *const Gfp, b: &Gfp, c: *mut Gfp, len: usize) {
    unsafe {
        // 4 registers as a group, that gives us 8 free v registers to use
        // t1: vl
        // t2: remaining element length
        // t3/t4: source address variables
        // t5: free variable
        // t6: destination variable
        // Only v0, v4, v8, v16, v24 and v28 are used. v8/v16 can be used as
        // 8-register group, rest are only used as 4-register group
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {a}",
            "mv t4, {b}",
            "mv t6, {c}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operands
            "vle256.v v0, (t3)",
            "vlse256.v v2, (t4), x0",
            // T = mul(a, b) => v8
            "vwmulu.vv v4, v0, v2",
            // Extract T[0..4] => v0
            "vnsrl.wx v0, v4, x0",
            // m = halfMul(T[0..4], np) => v4
            "vmul.vv v2, v0, v28",
            // t = mul(m, p2)=> v16
            "vwmulu.vv v8, v2, v30",
            // c = t + T = > v8, with carry in v0
            // Temporarily enlarging vlen to deal with bigger adds
            "vsetvli t1, t1, e512, m2",
            "vmadc.vv v0, v4, v8",
            "vadd.vv v4, v4, v8",
            "vsetvli t1, t1, e256, m2",
            // Extract c[4..8] => v4
            "li t5, 256",
            "vnsrl.wx v2, v4, t5",
            // gfpCarry using v4 in c[4..8], with carry in v0
            // c[4..8] - p2 => v16, with carry in v8
            "vmsbc.vv v4, v2, v30",
            "vsub.vv v8, v2, v30",
            // Combine carries
            "vmandnot.mm v0, v4, v0",
            // Select value, if carry is 1, use value in v4, otherwise use value in v16
            "vmerge.vvm v2, v8, v2, v0",
            // Store result
            "vse256.v v2, (t6)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            // t3/t4, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t3, t3, t1",
            "add t6, t6, t1",
            "blt x0, t2, 1b",
            len = in (reg) len,
            a = in (reg) a,
            b = in (reg) b as *const Gfp,
            c = in (reg) c,
        );
    }
}

#[inline(never)]
pub fn add_by_byte_index(a: &[Gfp], b: &[Gfp], a_index: &[u16], b_index: &[u16], c: &mut [Gfp]) {
    debug_assert_eq!(a_index.len(), b_index.len());
    debug_assert_eq!(b_index.len(), c.len());

    unsafe {
        // 8 registers as a group since add is simple and can do with less
        // registers
        // t1: vl
        // t2: remaining element length
        // t3/t4: source address variables
        // t5: free variable
        // t6: destination address
        // v0, v8, v16, v24 are used.
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {a_index}",
            "mv t4, {b_index}",
            "mv t6, {c}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operand indices
            "vle16.v v0, (t3)",
            "vle16.v v6, (t4)",
            // Load actual operands
            "mv t5, {a}",
            "vluxei16.v v2, (t5), v0",
            "mv t5, {b}",
            "vluxei16.v v4, (t5), v6",
            // Add operands together
            // c = a + b => v8, with carry in v0
            "vmadc.vv v0, v2, v4",
            "vadd.vv v2, v2, v4",
            // gfpCarry on c
            // c - p2 => v24, with carry in v16
            "vmsbc.vv v4, v2, v30",
            "vsub.vv v6, v2, v30",
            // Combine carries
            "vmandnot.mm v0, v4, v0",
            // Select value, if carry is 1, use value in v8 (c),
            // otherwise use value in v24 (c - p2)
            "vmerge.vvm v2, v6, v2, v0",
            // Store result
            "vse256.v v2, (t6)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            "add t3, t3, t1",
            "add t4, t4, t1",
            // t6, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t6, t6, t1",
            "blt x0, t2, 1b",
            len = in (reg) c.len(),
            a = in (reg) a.as_ptr(),
            b = in (reg) b.as_ptr(),
            c = in (reg) c.as_ptr(),
            a_index = in (reg) a_index.as_ptr(),
            b_index = in (reg) b_index.as_ptr(),
        );
    }
}

#[inline(never)]
pub fn mul_by_byte_index(a: &[Gfp], b: &[Gfp], a_index: &[u16], b_index: &[u16], c: &mut [Gfp]) {
    debug_assert_eq!(a_index.len(), b_index.len());
    debug_assert_eq!(b_index.len(), c.len());

    unsafe {
        // 4 registers as a group, that gives us 8 free v registers to use
        // t1: vl
        // t2: remaining element length
        // t3/t4: source address variables
        // t5: free variable
        // t6: destination variable
        // Only v0, v4, v8, v16, v24 and v28 are used. v8/v16 can be used as
        // 8-register group, rest are only used as 4-register group
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {a_index}",
            "mv t4, {b_index}",
            "mv t6, {c}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operand indices
            "vle16.v v12, (t3)",
            "vle16.v v14, (t4)",
            // Load actual operands
            "mv t5, {a}",
            "vluxei16.v v0, (t5), v12",
            "mv t5, {b}",
            "vluxei16.v v2, (t5), v14",
            // T = mul(a, b) => v8
            "vwmulu.vv v4, v0, v2",
            // Extract T[0..4] => v0
            "vnsrl.wx v0, v4, x0",
            // m = halfMul(T[0..4], np) => v4
            "vmul.vv v2, v0, v28",
            // t = mul(m, p2)=> v16
            "vwmulu.vv v8, v2, v30",
            // c = t + T = > v8, with carry in v0
            // Temporarily enlarging vlen to deal with bigger adds
            "vsetvli t1, t1, e512, m2",
            "vmadc.vv v0, v4, v8",
            "vadd.vv v4, v4, v8",
            "vsetvli t1, t1, e256, m2",
            // Extract c[4..8] => v4
            "li t5, 256",
            "vnsrl.wx v2, v4, t5",
            // gfpCarry using v4 in c[4..8], with carry in v0
            // c[4..8] - p2 => v16, with carry in v8
            "vmsbc.vv v4, v2, v30",
            "vsub.vv v8, v2, v30",
            // Combine carries
            "vmandnot.mm v0, v4, v0",
            // Select value, if carry is 1, use value in v4, otherwise use value in v16
            "vmerge.vvm v2, v8, v2, v0",
            // Store result
            "vse256.v v2, (t6)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            // t3 and t4 here stores byte-sized indices, which can be treated the same
            // way as count of elements.
            "sub t2, t2, t1",
            "add t3, t3, t1",
            "add t4, t4, t1",
            // t5 stores the address for result values, we will need to consider
            // element length as well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t6, t6, t1",
            "blt x0, t2, 1b",
            len = in (reg) c.len(),
            a = in (reg) a.as_ptr(),
            b = in (reg) b.as_ptr(),
            c = in (reg) c.as_ptr(),
            a_index = in (reg) a_index.as_ptr(),
            b_index = in (reg) b_index.as_ptr(),
        );
    }
}

#[inline(never)]
pub fn do_add(a: *const Gfp, b: *const Gfp, c: *mut Gfp, len: usize) {
    unsafe {
        // 8 registers as a group since add is simple and can do with less
        // registers
        // t1: vl
        // t2: remaining element length
        // t3/t4: source address variables
        // t5: free variable
        // t6: destination address
        // v0, v8, v16, v24 are used.
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {a}",
            "mv t4, {b}",
            "mv t6, {c}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load operands
            "vle256.v v2, (t3)",
            "vle256.v v4, (t4)",
            // Add operands together
            // c = a + b => v8, with carry in v0
            "vmadc.vv v0, v2, v4",
            "vadd.vv v2, v2, v4",
            // gfpCarry on c
            // c - p2 => v24, with carry in v16
            "vmsbc.vv v4, v2, v30",
            "vsub.vv v6, v2, v30",
            // Combine carries
            "vmandnot.mm v0, v4, v0",
            // Select value, if carry is 1, use value in v8 (c),
            // otherwise use value in v24 (c - p2)
            "vmerge.vvm v2, v6, v2, v0",
            // Store result
            "vse256.v v2, (t6)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            // t3/t4, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t3, t3, t1",
            "add t4, t4, t1",
            "add t6, t6, t1",
            "blt x0, t2, 1b",
            len = in (reg) len,
            a = in (reg) a,
            b = in (reg) b,
            c = in (reg) c,
        );
    }
}

#[inline(never)]
pub fn do_sub(a: *const Gfp, b: *const Gfp, c: *mut Gfp, len: usize) {
    unsafe {
        // 8 registers as a group since add is simple and can do with less
        // registers
        // t1: vl
        // t2: remaining element length
        // t3/t4: source address variables
        // t5: free variable
        // t6: destination address
        // v0, v8, v16, v24 are used.
        rvv_asm!(
            "mv t2, {len}",
            "mv t3, {a}",
            "mv t4, {b}",
            "mv t6, {c}",
            "1:",
            "vsetvli t1, t2, e256, m2",
            // Load a into v8, b into v16
            "vle256.v v2, (t3)",
            "vle256.v v4, (t4)",
            // c = a - b => v8, carry is put in v0
            "vmsbc.vv v0, v2, v4",
            "vsub.vv v2, v2, v4",
            // Clear v16 to all zeros
            "vxor.vv v4, v4, v4",
            // If carry is present, select p2 in v24, otherwise select 0 in v16
            "vmerge.vvm v4, v4, v30, v0",
            // Possibly add p2 to final result(when carry is present)
            "vadd.vv v2, v2, v4",
            // Store result
            "vse256.v v2, (t6)",
            // Update t2/t3/t4, start the next loop if required, t2 contains the count
            // of elements, so we do substraction using value in t1 directly.
            "sub t2, t2, t1",
            // t3/t4, on the other hand, stores the address, we will need to consider
            // element length asl well. A single element is 32 bytes, a shift left
            // by 5 on t1 will do the task
            "slli t1, t1, 5",
            "add t3, t3, t1",
            "add t4, t4, t1",
            "blt x0, t2, 1b",
            len = in (reg) len,
            a = in (reg) a,
            b = in (reg) b,
            c = in (reg) c,
        );
    }
}
