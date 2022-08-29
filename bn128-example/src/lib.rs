#![no_std]
#![feature(asm)]

pub mod arith;
pub mod ethereum;
mod fields;
mod groups;
pub mod rvv_crypto;
pub mod rvv_impl;

use crate::fields::FieldElement;
use crate::groups::{G1Params, G2Params, GroupElement, GroupParams};
use core::ops::{Add, Mul, Neg, Sub};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Fr(fields::Fr);

impl Fr {
    pub fn zero() -> Self {
        Fr(fields::Fr::zero())
    }
    pub fn one() -> Self {
        Fr(fields::Fr::one())
    }
    pub fn pow(&self, exp: Fr) -> Self {
        Fr(self.0.pow(exp.0))
    }
    pub fn from_str(s: &str) -> Option<Self> {
        fields::Fr::from_str(s).map(|e| Fr(e))
    }
    pub fn inverse(&self) -> Option<Self> {
        self.0.inverse().map(|e| Fr(e))
    }
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
    pub fn interpret(buf: &[u8; 64]) -> Fr {
        Fr(fields::Fr::interpret(buf))
    }
    pub fn from_slice(slice: &[u8]) -> Result<Self, FieldError> {
        arith::U256::from_slice(slice)
            .map_err(|_| FieldError::InvalidSliceLength) // todo: maybe more sensful error handling
            .map(|x| Fr::new_mul_factor(x))
    }
    pub fn to_big_endian(&self, slice: &mut [u8]) -> Result<(), FieldError> {
        self.0
            .raw()
            .to_big_endian(slice)
            .map_err(|_| FieldError::InvalidSliceLength)
    }
    pub fn new(val: arith::U256) -> Option<Self> {
        fields::Fr::new(val).map(|x| Fr(x))
    }
    pub fn new_mul_factor(val: arith::U256) -> Self {
        Fr(fields::Fr::new_mul_factor(val))
    }
    pub fn into_u256(self) -> arith::U256 {
        (self.0).into()
    }
    pub fn set_bit(&mut self, bit: usize, to: bool) {
        self.0.set_bit(bit, to);
    }
}

impl Add<Fr> for Fr {
    type Output = Fr;

    fn add(self, other: Fr) -> Fr {
        Fr(self.0 + other.0)
    }
}

impl Sub<Fr> for Fr {
    type Output = Fr;

    fn sub(self, other: Fr) -> Fr {
        Fr(self.0 - other.0)
    }
}

impl Neg for Fr {
    type Output = Fr;

    fn neg(self) -> Fr {
        Fr(-self.0)
    }
}

impl Mul for Fr {
    type Output = Fr;

    fn mul(self, other: Fr) -> Fr {
        Fr(self.0 * other.0)
    }
}

#[derive(Debug)]
pub enum FieldError {
    InvalidSliceLength,
    InvalidU512Encoding,
    NotMember,
}

#[derive(Debug)]
pub enum CurveError {
    InvalidEncoding,
    NotMember,
    Field(FieldError),
    ToAffineConversion,
}

impl From<FieldError> for CurveError {
    fn from(fe: FieldError) -> Self {
        CurveError::Field(fe)
    }
}

pub use crate::groups::Error as GroupError;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Fq(fields::Fq);

impl Fq {
    pub fn zero() -> Self {
        Fq(fields::Fq::zero())
    }
    pub fn one() -> Self {
        Fq(fields::Fq::one())
    }
    pub fn pow(&self, exp: Fq) -> Self {
        Fq(self.0.pow(exp.0))
    }
    pub fn from_str(s: &str) -> Option<Self> {
        fields::Fq::from_str(s).map(|e| Fq(e))
    }
    pub fn inverse(&self) -> Option<Self> {
        self.0.inverse().map(|e| Fq(e))
    }
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
    pub fn interpret(buf: &[u8; 64]) -> Fq {
        Fq(fields::Fq::interpret(buf))
    }
    pub fn from_slice(slice: &[u8]) -> Result<Self, FieldError> {
        arith::U256::from_slice(slice)
            .map_err(|_| FieldError::InvalidSliceLength) // todo: maybe more sensful error handling
            .and_then(|x| fields::Fq::new(x).ok_or(FieldError::NotMember))
            .map(|x| Fq(x))
    }
    pub fn to_big_endian(&self, slice: &mut [u8]) -> Result<(), FieldError> {
        let mut a: arith::U256 = self.0.into();
        // convert from Montgomery representation
        a.mul(
            &fields::Fq::one().raw(),
            &fields::Fq::modulus(),
            self.0.inv(),
        );
        a.to_big_endian(slice)
            .map_err(|_| FieldError::InvalidSliceLength)
    }
    pub fn from_u256(u256: arith::U256) -> Result<Self, FieldError> {
        Ok(Fq(fields::Fq::new(u256).ok_or(FieldError::NotMember)?))
    }
    pub fn into_u256(self) -> arith::U256 {
        (self.0).into()
    }
    pub fn modulus() -> arith::U256 {
        fields::Fq::modulus()
    }

    pub fn sqrt(&self) -> Option<Self> {
        self.0.sqrt().map(Fq)
    }
}

impl Add<Fq> for Fq {
    type Output = Fq;

    fn add(self, other: Fq) -> Fq {
        Fq(self.0 + other.0)
    }
}

impl Sub<Fq> for Fq {
    type Output = Fq;

    fn sub(self, other: Fq) -> Fq {
        Fq(self.0 - other.0)
    }
}

impl Neg for Fq {
    type Output = Fq;

    fn neg(self) -> Fq {
        Fq(-self.0)
    }
}

impl Mul for Fq {
    type Output = Fq;

    fn mul(self, other: Fq) -> Fq {
        Fq(self.0 * other.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Fq2(fields::Fq2);

impl Fq2 {
    pub fn one() -> Fq2 {
        Fq2(fields::Fq2::one())
    }

    pub fn i() -> Fq2 {
        Fq2(fields::Fq2::i())
    }

    pub fn zero() -> Fq2 {
        Fq2(fields::Fq2::zero())
    }

    /// Initalizes new F_q2(a + bi, a is real coeff, b is imaginary)
    pub fn new(a: Fq, b: Fq) -> Fq2 {
        Fq2(fields::Fq2::new(a.0, b.0))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn pow(&self, exp: arith::U256) -> Self {
        Fq2(self.0.pow(exp))
    }

    pub fn real(&self) -> Fq {
        Fq(*self.0.real())
    }

    pub fn imaginary(&self) -> Fq {
        Fq(*self.0.imaginary())
    }

    pub fn sqrt(&self) -> Option<Self> {
        self.0.sqrt().map(Fq2)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, FieldError> {
        let u512 = arith::U512::from_slice(bytes).map_err(|_| FieldError::InvalidU512Encoding)?;
        let (res, c0) = u512.divrem(&Fq::modulus());
        Ok(Fq2::new(
            Fq::from_u256(c0).map_err(|_| FieldError::NotMember)?,
            Fq::from_u256(res.ok_or(FieldError::NotMember)?).map_err(|_| FieldError::NotMember)?,
        ))
    }
}

impl Add<Fq2> for Fq2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Fq2(self.0 + other.0)
    }
}

impl Sub<Fq2> for Fq2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Fq2(self.0 - other.0)
    }
}

impl Neg for Fq2 {
    type Output = Self;

    fn neg(self) -> Self {
        Fq2(-self.0)
    }
}

impl Mul for Fq2 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Fq2(self.0 * other.0)
    }
}

pub trait Group:
    Send
    + Sync
    + Copy
    + Clone
    + PartialEq
    + Eq
    + Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Neg<Output = Self>
    + Mul<Fr, Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn is_zero(&self) -> bool;
    fn normalize(&mut self);
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct G1(groups::G1);

impl G1 {
    pub fn new(x: Fq, y: Fq, z: Fq) -> Self {
        G1(groups::G1::new(x.0, y.0, z.0))
    }

    pub fn x(&self) -> Fq {
        Fq(self.0.x().clone())
    }

    pub fn set_x(&mut self, x: Fq) {
        *self.0.x_mut() = x.0
    }

    pub fn y(&self) -> Fq {
        Fq(self.0.y().clone())
    }

    pub fn set_y(&mut self, y: Fq) {
        *self.0.y_mut() = y.0
    }

    pub fn z(&self) -> Fq {
        Fq(self.0.z().clone())
    }

    pub fn set_z(&mut self, z: Fq) {
        *self.0.z_mut() = z.0
    }

    pub fn b() -> Fq {
        Fq(G1Params::coeff_b())
    }

    pub fn from_compressed(bytes: &[u8]) -> Result<Self, CurveError> {
        if bytes.len() != 33 {
            return Err(CurveError::InvalidEncoding);
        }

        let sign = bytes[0];
        let fq = Fq::from_slice(&bytes[1..])?;
        let x = fq;
        let y_squared = (fq * fq * fq) + Self::b();

        let mut y = y_squared.sqrt().ok_or(CurveError::NotMember)?;

        if sign == 2 && y.into_u256().get_bit(0).expect("bit 0 always exist; qed") {
            y = y.neg();
        } else if sign == 3 && !y.into_u256().get_bit(0).expect("bit 0 always exist; qed") {
            y = y.neg();
        } else if sign != 3 && sign != 2 {
            return Err(CurveError::InvalidEncoding);
        }
        AffineG1::new(x, y)
            .map_err(|_| CurveError::NotMember)
            .map(Into::into)
    }
}

impl Group for G1 {
    fn zero() -> Self {
        G1(groups::G1::zero())
    }
    fn one() -> Self {
        G1(groups::G1::one())
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
    fn normalize(&mut self) {
        let new = match self.0.to_affine() {
            Some(a) => a,
            None => return,
        };

        self.0 = new.to_jacobian();
    }
}

impl Add<G1> for G1 {
    type Output = G1;

    fn add(self, other: G1) -> G1 {
        G1(self.0 + other.0)
    }
}

impl Sub<G1> for G1 {
    type Output = G1;

    fn sub(self, other: G1) -> G1 {
        G1(self.0 - other.0)
    }
}

impl Neg for G1 {
    type Output = G1;

    fn neg(self) -> G1 {
        G1(-self.0)
    }
}

impl Mul<Fr> for G1 {
    type Output = G1;

    fn mul(self, other: Fr) -> G1 {
        G1(self.0 * other.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct AffineG1(groups::AffineG1);

impl AffineG1 {
    pub fn new(x: Fq, y: Fq) -> Result<Self, GroupError> {
        Ok(AffineG1(groups::AffineG1::new(x.0, y.0)?))
    }

    pub fn x(&self) -> Fq {
        Fq(self.0.x().clone())
    }

    pub fn set_x(&mut self, x: Fq) {
        *self.0.x_mut() = x.0
    }

    pub fn y(&self) -> Fq {
        Fq(self.0.y().clone())
    }

    pub fn set_y(&mut self, y: Fq) {
        *self.0.y_mut() = y.0
    }

    pub fn from_jacobian(g1: G1) -> Option<Self> {
        g1.0.to_affine().map(|x| AffineG1(x))
    }
}

impl From<AffineG1> for G1 {
    fn from(affine: AffineG1) -> Self {
        G1(affine.0.to_jacobian())
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct G2(groups::G2);

impl G2 {
    pub fn new(x: Fq2, y: Fq2, z: Fq2) -> Self {
        G2(groups::G2::new(x.0, y.0, z.0))
    }

    pub fn x(&self) -> Fq2 {
        Fq2(self.0.x().clone())
    }

    pub fn set_x(&mut self, x: Fq2) {
        *self.0.x_mut() = x.0
    }

    pub fn y(&self) -> Fq2 {
        Fq2(self.0.y().clone())
    }

    pub fn set_y(&mut self, y: Fq2) {
        *self.0.y_mut() = y.0
    }

    pub fn z(&self) -> Fq2 {
        Fq2(self.0.z().clone())
    }

    pub fn set_z(&mut self, z: Fq2) {
        *self.0.z_mut() = z.0
    }

    pub fn b() -> Fq2 {
        Fq2(G2Params::coeff_b())
    }

    pub fn from_compressed(bytes: &[u8]) -> Result<Self, CurveError> {
        if bytes.len() != 65 {
            return Err(CurveError::InvalidEncoding);
        }

        let sign = bytes[0];
        let x = Fq2::from_slice(&bytes[1..])?;

        let y_squared = (x * x * x) + G2::b();
        let y = y_squared.sqrt().ok_or(CurveError::NotMember)?;
        let y_neg = -y;

        let y_gt = y.0.to_u512() > y_neg.0.to_u512();

        let e_y = if sign == 10 {
            if y_gt {
                y_neg
            } else {
                y
            }
        } else if sign == 11 {
            if y_gt {
                y
            } else {
                y_neg
            }
        } else {
            return Err(CurveError::InvalidEncoding);
        };

        AffineG2::new(x, e_y)
            .map_err(|_| CurveError::NotMember)
            .map(Into::into)
    }
}

impl Group for G2 {
    fn zero() -> Self {
        G2(groups::G2::zero())
    }
    fn one() -> Self {
        G2(groups::G2::one())
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
    fn normalize(&mut self) {
        let new = match self.0.to_affine() {
            Some(a) => a,
            None => return,
        };

        self.0 = new.to_jacobian();
    }
}

impl Add<G2> for G2 {
    type Output = G2;

    fn add(self, other: G2) -> G2 {
        G2(self.0 + other.0)
    }
}

impl Sub<G2> for G2 {
    type Output = G2;

    fn sub(self, other: G2) -> G2 {
        G2(self.0 - other.0)
    }
}

impl Neg for G2 {
    type Output = G2;

    fn neg(self) -> G2 {
        G2(-self.0)
    }
}

impl Mul<Fr> for G2 {
    type Output = G2;

    fn mul(self, other: Fr) -> G2 {
        G2(self.0 * other.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Gt(fields::Fq12);

impl Gt {
    pub fn one() -> Self {
        Gt(fields::Fq12::one())
    }
    pub fn pow(&self, exp: Fr) -> Self {
        Gt(self.0.pow(exp.0))
    }
    pub fn inverse(&self) -> Option<Self> {
        self.0.inverse().map(Gt)
    }
    pub fn final_exponentiation(&self) -> Option<Self> {
        self.0.final_exponentiation().map(Gt)
    }
}

impl Mul<Gt> for Gt {
    type Output = Gt;

    fn mul(self, other: Gt) -> Gt {
        Gt(self.0 * other.0)
    }
}

pub fn pairing(p: G1, q: G2) -> Gt {
    Gt(groups::pairing(&p.0, &q.0))
}

pub fn pairing_batch(pairs: &[(G1, G2)]) -> Gt {
    let mut ps = [groups::G1::default(); 16];
    let mut qs = [groups::G2::default(); 16];
    for (i, (p, q)) in pairs.iter().enumerate() {
        ps[i] = p.0;
        qs[i] = q.0;
    }
    Gt(groups::pairing_batch(
        &ps[0..pairs.len()],
        &qs[0..pairs.len()],
    ))
}

pub fn miller_loop_batch(pairs: &[(G2, G1)]) -> Result<Gt, CurveError> {
    let mut ps = [groups::G2Precomp::default(); 16];
    let mut qs = [groups::AffineG::<groups::G1Params>::default(); 16];
    for (i, (p, q)) in pairs.iter().enumerate() {
        ps[i] =
            p.0.to_affine()
                .ok_or(CurveError::ToAffineConversion)?
                .precompute();
        qs[i] = q.0.to_affine().ok_or(CurveError::ToAffineConversion)?;
    }
    Ok(Gt(groups::miller_loop_batch(&ps, &qs)))
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct AffineG2(groups::AffineG2);

impl AffineG2 {
    pub fn new(x: Fq2, y: Fq2) -> Result<Self, GroupError> {
        Ok(AffineG2(groups::AffineG2::new(x.0, y.0)?))
    }

    pub fn x(&self) -> Fq2 {
        Fq2(self.0.x().clone())
    }

    pub fn set_x(&mut self, x: Fq2) {
        *self.0.x_mut() = x.0
    }

    pub fn y(&self) -> Fq2 {
        Fq2(self.0.y().clone())
    }

    pub fn set_y(&mut self, y: Fq2) {
        *self.0.y_mut() = y.0
    }

    pub fn from_jacobian(g2: G2) -> Option<Self> {
        g2.0.to_affine().map(|x| AffineG2(x))
    }
}

impl From<AffineG2> for G2 {
    fn from(affine: AffineG2) -> Self {
        G2(affine.0.to_jacobian())
    }
}
