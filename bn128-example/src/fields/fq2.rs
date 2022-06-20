extern crate alloc;

use crate::arith::{U256, U512};
use crate::fields::{const_fq, FieldElement, Fq};
use core::mem;
use core::ops::{Add, Mul, Neg, Sub};

use super::fp::{batch_mul, batch_mul2};

#[inline]
fn fq_non_residue() -> Fq {
    // (q - 1) is a quadratic nonresidue in Fq
    // 21888242871839275222246405745257275088696311157297823662689037894645226208582
    const_fq([
        0x68c3488912edefaa,
        0x8d087f6872aabf4f,
        0x51e1a24709081231,
        0x2259d6b14729c0fa,
    ])
}

#[inline]
pub fn fq2_nonresidue() -> Fq2 {
    Fq2::new(
        const_fq([
            0xf60647ce410d7ff7,
            0x2f3d6f4dd31bd011,
            0x2943337e3940c6d1,
            0x1d9598e8a7e39857,
        ]),
        const_fq([
            0xd35d438dc58f0d9d,
            0x0a78eb28f5c70b3d,
            0x666ea36f7879462c,
            0x0e0a77c19a07df2f,
        ]),
    )
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Fq2 {
    pub c0: Fq,
    pub c1: Fq,
}

impl Fq2 {
    pub fn new(c0: Fq, c1: Fq) -> Self {
        Fq2 { c0: c0, c1: c1 }
    }

    pub fn scale(&self, by: Fq) -> Self {
        if cfg!(feature = "use_rvv-asm") {
            let (c0, c1) = batch_mul2(self.c0, self.c1, by, by);
            Fq2 { c0, c1 }
        } else {
            Fq2 {
                c0: self.c0 * by,
                c1: self.c1 * by,
            }
        }
    }

    pub fn mul_by_nonresidue(&self) -> Self {
        *self * fq2_nonresidue()
    }

    pub fn frobenius_map(&self, power: usize) -> Self {
        if power % 2 == 0 {
            *self
        } else {
            Fq2 {
                c0: self.c0,
                c1: self.c1 * fq_non_residue(),
            }
        }
    }

    pub fn real(&self) -> &Fq {
        &self.c0
    }

    pub fn imaginary(&self) -> &Fq {
        &self.c1
    }
}

impl FieldElement for Fq2 {
    fn zero() -> Self {
        Fq2 {
            c0: Fq::zero(),
            c1: Fq::zero(),
        }
    }

    fn one() -> Self {
        Fq2 {
            c0: Fq::one(),
            c1: Fq::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.c0.is_zero() && self.c1.is_zero()
    }

    fn squared(&self) -> Self {
        // Devegili OhEig Scott Dahab
        //     Multiplication and Squaring on Pairing-Friendly Fields.pdf
        //     Section 3 (Complex squaring)
        if cfg!(feature = "use_rvv_asm") {
            let (a, b) = batch_mul2(
                self.c0,
                self.c1 * fq_non_residue() + self.c0,
                self.c1,
                self.c0 + self.c1,
            );

            Fq2 {
                c0: b - a - a * fq_non_residue(),
                c1: a + a,
            }
        } else {
            let ab = self.c0 * self.c1;

            Fq2 {
                c0: (self.c1 * fq_non_residue() + self.c0) * (self.c0 + self.c1)
                    - ab
                    - ab * fq_non_residue(),
                c1: ab + ab,
            }
        }
    }

    fn inverse(self) -> Option<Self> {
        // "High-Speed Software Implementation of the Optimal Ate Pairing
        // over Barreto–Naehrig Curves"; Algorithm 8
        if cfg!(feature = "use_rvv_asm") {
            match (self.c0.squared() - (self.c1.squared() * fq_non_residue())).inverse() {
                Some(t) => {
                    let (a, b) = batch_mul2(self.c0, self.c1, t, t);
                    Some(Fq2 { c0: a, c1: -b })
                }
                None => None,
            }
        } else {
            match (self.c0.squared() - (self.c1.squared() * fq_non_residue())).inverse() {
                Some(t) => Some(Fq2 {
                    c0: self.c0 * t,
                    c1: -(self.c1 * t),
                }),
                None => None,
            }
        }
    }
}

impl Mul for Fq2 {
    type Output = Fq2;

    fn mul(self, other: Fq2) -> Fq2 {
        // Devegili OhEig Scott Dahab
        //     Multiplication and Squaring on Pairing-Friendly Fields.pdf
        //     Section 3 (Karatsuba)
        if cfg!(feature = "use_rvv_asm") {
            let mut result: [Fq; 4] = [Fq::zero(); 4];
            let lhs = [self.c0, self.c1, self.c0, self.c1];
            let rhs = [other.c0, other.c1, other.c1, other.c0];
            batch_mul(&lhs, &rhs, &mut result);
            Fq2 {
                c0: result[0] - result[1],
                c1: result[2] + result[3],
            }
        } else {
            let aa = self.c0 * other.c0;
            let bb = self.c1 * other.c1;

            Fq2 {
                c0: bb * fq_non_residue() + aa,
                c1: (self.c0 + self.c1) * (other.c0 + other.c1) - aa - bb,
            }
        }
    }
}

impl Sub for Fq2 {
    type Output = Fq2;

    fn sub(self, other: Fq2) -> Fq2 {
        Fq2 {
            c0: self.c0 - other.c0,
            c1: self.c1 - other.c1,
        }
    }
}

impl Add for Fq2 {
    type Output = Fq2;

    fn add(self, other: Fq2) -> Fq2 {
        Fq2 {
            c0: self.c0 + other.c0,
            c1: self.c1 + other.c1,
        }
    }
}

impl Neg for Fq2 {
    type Output = Fq2;

    fn neg(self) -> Fq2 {
        Fq2 {
            c0: -self.c0,
            c1: -self.c1,
        }
    }
}

lazy_static::lazy_static! {
    static ref FQ: U256 = U256::from([
        0x3c208c16d87cfd47,
        0x97816a916871ca8d,
        0xb85045b68181585d,
        0x30644e72e131a029
    ]);

    static ref FQ_MINUS3_DIV4: Fq =
        Fq::new(3.into()).expect("3 is a valid field element and static; qed").neg() *
        Fq::new(4.into()).expect("4 is a valid field element and static; qed").inverse()
        .expect("4 has inverse in Fq and is static; qed");

    static ref FQ_MINUS1_DIV2: Fq =
        Fq::new(1.into()).expect("1 is a valid field element and static; qed").neg() *
        Fq::new(2.into()).expect("2 is a valid field element and static; qed").inverse()
            .expect("2 has inverse in Fq and is static; qed");
}

impl Fq2 {
    pub fn i() -> Fq2 {
        Fq2::new(Fq::zero(), Fq::one())
    }

    pub fn sqrt(&self) -> Option<Self> {
        let a1 = self.pow::<U256>((*FQ_MINUS3_DIV4).into());
        let a1a = a1 * *self;
        let alpha = a1 * a1a;
        let a0 = alpha.pow(*FQ) * alpha;

        if a0 == Fq2::one().neg() {
            return None;
        }

        if alpha == Fq2::one().neg() {
            Some(Self::i() * a1a)
        } else {
            let b = (alpha + Fq2::one()).pow::<U256>((*FQ_MINUS1_DIV2).into());
            Some(b * a1a)
        }
    }

    pub fn to_u512(&self) -> U512 {
        let c0: U256 = (*self.real()).into();
        let c1: U256 = (*self.imaginary()).into();

        U512::new(&c1, &c0, &FQ)
    }
}

#[test]
fn sqrt_fq2() {
    // from zcash test_proof.cpp
    let x1 = Fq2::new(
        Fq::from_str(
            "12844195307879678418043983815760255909500142247603239203345049921980497041944",
        )
        .unwrap(),
        Fq::from_str(
            "7476417578426924565731404322659619974551724117137577781074613937423560117731",
        )
        .unwrap(),
    );

    let x2 = Fq2::new(
        Fq::from_str(
            "3345897230485723946872934576923485762803457692345760237495682347502347589474",
        )
        .unwrap(),
        Fq::from_str(
            "1234912378405347958234756902345768290345762348957605678245967234857634857676",
        )
        .unwrap(),
    );

    assert_eq!(x2.sqrt().unwrap(), x1);

    // i is sqrt(-1)
    assert_eq!(Fq2::one().neg().sqrt().unwrap(), Fq2::i(),);

    // no sqrt for (1 + 2i)
    assert!(
        Fq2::new(Fq::from_str("1").unwrap(), Fq::from_str("2").unwrap())
            .sqrt()
            .is_none()
    );
}

pub fn batch_mul3(
    lhs1: Fq2,
    lhs2: Fq2,
    lhs3: Fq2,

    rhs1: Fq2,
    rhs2: Fq2,
    rhs3: Fq2,
) -> (Fq2, Fq2, Fq2) {
    let lhs = [lhs1.c0, lhs1.c1, lhs2.c0, lhs2.c1, lhs3.c0, lhs3.c1];
    let rhs = [rhs1.c0, rhs1.c1, rhs2.c0, rhs2.c1, rhs3.c0, rhs3.c1];
    let mut result: [Fq; 6] = unsafe { mem::MaybeUninit::uninit().assume_init() };

    batch_mul(&lhs, &rhs, &mut result);

    let mut result2: [Fq; 6] = unsafe { mem::MaybeUninit::uninit().assume_init() };

    let lhs = [
        result[1],
        lhs1.c0 + lhs1.c1,
        result[3],
        lhs2.c0 + lhs2.c1,
        result[5],
        lhs3.c0 + lhs3.c1,
    ];
    let rhs = [
        fq_non_residue(),
        rhs1.c0 + rhs1.c1,
        fq_non_residue(),
        rhs2.c0 + rhs2.c1,
        fq_non_residue(),
        rhs3.c0 + rhs3.c1,
    ];
    batch_mul(&lhs, &rhs, &mut result2);
    let [a1, b1, a2, b2, a3, b3] = result2;

    let r1 = Fq2 {
        c0: a1 + result[0],
        c1: b1 - result[0] - result[1],
    };

    let r2 = Fq2 {
        c0: a2 + result[2],
        c1: b2 - result[2] - result[3],
    };

    let r3 = Fq2 {
        c0: a3 + result[4],
        c1: b3 - result[4] - result[5],
    };

    (r1, r2, r3)
}
