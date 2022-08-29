use super::{
    casts::*,
    constants::*,
    gfp::{self, Gfp},
    gfp6::Gfp6,
    macros::gfp_ops_impl,
};
use crate::arith::U256;
use core::mem::MaybeUninit;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

fn bits(a: &U256) -> usize {
    256 - a.bits().take_while(|b| !b).count()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gfp12(pub [Gfp6; 2]);

impl Gfp12 {
    pub fn one() -> Self {
        let mut a: Self = unsafe { MaybeUninit::uninit().assume_init() };
        a.set_one();
        a
    }

    pub fn x(&self) -> &Gfp6 {
        &self.0[0]
    }

    pub fn y(&self) -> &Gfp6 {
        &self.0[1]
    }

    pub fn x_mut(&mut self) -> &mut Gfp6 {
        &mut self.0[0]
    }

    pub fn y_mut(&mut self) -> &mut Gfp6 {
        &mut self.0[1]
    }

    pub fn x_slice_mut(&mut self) -> &mut [Gfp6] {
        &mut self.0[0..1]
    }

    pub fn y_slice_mut(&mut self) -> &mut [Gfp6] {
        &mut self.0[1..2]
    }

    pub fn x_slice(&mut self) -> &[Gfp6] {
        &self.0[0..1]
    }

    pub fn y_slice(&mut self) -> &[Gfp6] {
        &self.0[1..2]
    }

    pub fn set_x(&mut self, x: &Gfp6) -> &mut Self {
        self.0[0].set(x);
        self
    }

    pub fn set_y(&mut self, y: &Gfp6) -> &mut Self {
        self.0[1].set(y);
        self
    }

    pub fn set(&mut self, other: &Gfp12) -> &mut Self {
        self.set_x(other.x());
        self.set_y(other.y());
        self
    }

    pub fn set_zero(&mut self) -> &mut Self {
        self.x_mut().set_zero();
        self.y_mut().set_zero();
        self
    }

    pub fn set_one(&mut self) -> &mut Self {
        self.x_mut().set_zero();
        self.y_mut().set_one();
        self
    }

    pub fn is_zero(&self) -> bool {
        self.x().is_zero() && self.y().is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.x().is_zero() && self.y().is_one()
    }

    pub fn conjugate(&mut self) -> &mut Self {
        self.x_mut().neg_ref();
        self
    }

    pub fn conjugate_to(&self) -> Self {
        Gfp12([self.x().neg_to(), self.y().clone()])
    }

    pub fn neg_ref(&mut self) -> &mut Self {
        self.x_mut().neg_ref();
        self.y_mut().neg_ref();
        self
    }

    pub fn neg_to(&self) -> Self {
        Self([self.x().neg_to(), self.y().neg_to()])
    }

    pub fn frobenius(&mut self) -> &mut Self {
        self.x_mut()
            .frobenius()
            .mul_scalar(&constant_to_gfp2(&XI_TO_P_MINUS1_OVER6));
        self.y_mut().frobenius();
        self
    }

    pub fn frobenius_to(&self) -> Self {
        let mut r = Self([self.x().frobenius_to(), self.y().frobenius_to()]);
        r.x_mut()
            .mul_scalar(&constant_to_gfp2(&XI_TO_P_MINUS1_OVER6));
        r
    }

    pub fn frobenius_p2(&mut self) -> &mut Self {
        self.x_mut()
            .frobenius_p2()
            .mul_gfp(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER6));
        self.y_mut().frobenius_p2();
        self
    }

    pub fn frobenius_p2_to(&self) -> Self {
        let mut r = Self([self.x().frobenius_p2_to(), self.y().frobenius_p2_to()]);
        r.x_mut().mul_gfp(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER6));
        r
    }

    pub fn frobenius_p4(&mut self) -> &mut Self {
        self.x_mut()
            .frobenius_p4()
            .mul_gfp(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER3));
        self.y_mut().frobenius_p4();
        self
    }

    pub fn add_ref(&mut self, b: &Gfp12) -> &mut Self {
        let dst = gfp6_to_gfp_slice_mut(&mut self.0);
        let src = gfp6_to_gfp_slice(&b.0);

        gfp::add_mov(dst, src);
        self
    }

    pub fn add_to(a: &Gfp12, b: &Gfp12) -> Self {
        let mut r: Gfp12 = unsafe { MaybeUninit::uninit().assume_init() };
        r.0[0] = a.x() + b.x();
        r.0[1] = a.y() + b.y();
        r
    }

    pub fn sub_ref(&mut self, b: &Gfp12) -> &mut Self {
        self.x_mut().sub_ref(&b.x());
        self.y_mut().sub_ref(&b.y());
        self
    }

    pub fn sub_to(a: &Gfp12, b: &Gfp12) -> Self {
        let mut r: Gfp12 = unsafe { MaybeUninit::uninit().assume_init() };
        r.0[0] = a.x() - b.x();
        r.0[1] = a.y() - b.y();
        r
    }

    pub fn mul_ref(&mut self, b: &Gfp12) -> &mut Self {
        let t = self.y() * b.x();
        let mut t2 = self.x() * b.x();
        t2.mul_tau();

        self.mul_scalar(b.y());
        self.x_mut().add_ref(&t);
        self.y_mut().add_ref(&t2);
        self
    }

    pub fn mul_to(a: &Gfp12, b: &Gfp12) -> Self {
        let t = a.y() * b.x();
        let mut t2 = a.x() * b.x();
        t2.mul_tau();

        let mut r = a.mul_scalar_to(b.y());
        r.x_mut().add_ref(&t);
        r.y_mut().add_ref(&t2);
        r
    }

    pub fn mul_scalar(&mut self, b: &Gfp6) -> &mut Self {
        self.x_mut().mul_ref(b);
        self.y_mut().mul_ref(b);
        self
    }

    pub fn mul_scalar_to(&self, b: &Gfp6) -> Self {
        Gfp12([Gfp6::mul_to(self.x(), b), Gfp6::mul_to(self.y(), b)])
    }

    pub fn exp_to(&self, power: &U256) -> Self {
        let mut sum = Gfp12::one();
        let mut t = Gfp12::default();

        for i in (0..bits(power)).rev() {
            sum.square_to_mut(&mut t);
            if power.get_bit(i).unwrap_or(false) {
                sum.set(&t).mul_ref(self);
            } else {
                sum.set(&t);
            }
        }

        sum
    }

    #[inline(always)]
    pub fn exp(&mut self, power: &U256) -> &mut Self {
        *self = self.exp_to(power);
        self
    }

    pub fn square(&mut self) -> &mut Self {
        let mut t = self.x().mul_tau_to();
        t.add_ref(self.y());
        let v0 = [self.x() * self.y()];

        {
            gfp::do_add(
                gfp6_to_gfp_slice(self.x_slice()).as_ptr(),
                gfp6_to_gfp_slice(self.y_slice()).as_ptr(),
                gfp6_to_gfp_slice_mut(self.y_slice_mut()).as_mut_ptr(),
                gfp6_to_gfp_slice_mut(self.y_slice_mut()).len(),
            );
        }
        self.y_mut().mul_ref(&t).sub_ref(&v0[0]);
        t.set(&v0[0]).mul_tau();
        self.y_mut().sub_ref(&t);

        {
            let src = gfp6_to_gfp_slice(&v0);
            let dst = gfp6_to_gfp_slice_mut(self.x_slice_mut());

            gfp::double_to(src, dst);
        }
        self
    }

    pub fn square_to_mut(&self, target: &mut Self) {
        let mut t = self.x().mul_tau_to();
        t.add_ref(self.y());
        let v0 = [self.x() * self.y()];

        Gfp6::add_to_mut(self.x(), self.y(), target.y_mut());
        target.y_mut().mul_ref(&t).sub_ref(&v0[0]);
        t.set(&v0[0]).mul_tau();
        target.y_mut().sub_ref(&t);

        {
            let src = gfp6_to_gfp_slice(&v0);
            let dst = gfp6_to_gfp_slice_mut(target.x_slice_mut());

            gfp::double_to(src, dst);
        }
    }

    pub fn invert(&mut self) -> &mut Self {
        let mut t1 = self.x().square_to();
        let mut t2 = self.y().square_to();

        t1.mul_tau();
        t2.sub_ref(&t1);
        t2.invert();

        self.x_mut().neg_ref();
        self.mul_scalar(&t2);
        self
    }

    pub fn invert_to(&self) -> Self {
        let mut t1 = self.x().square_to();
        let mut t2 = self.y().square_to();

        t1.mul_tau();
        t2.sub_ref(&t1);
        t2.invert();

        let mut r = Gfp12([self.x().neg_to(), self.y().clone()]);
        r.mul_scalar(&t2);
        r
    }

    pub fn final_exponentiation(&mut self) -> &mut Self {
        // p^6-Frobenus
        let mut t1 = Gfp12([self.x().neg_to(), self.y().clone()]);

        let inv = self.invert_to();
        t1.mul_ref(&inv);

        let t2 = t1.frobenius_p2_to();
        t1.mul_ref(&t2);

        let fp = t1.frobenius_to();
        let fp2 = t1.frobenius_p2_to();
        let fp3 = fp2.frobenius_to();

        let fu = t1.exp_to(&U.into());
        let fu2 = fu.exp_to(&U.into());
        let fu3 = fu2.exp_to(&U.into());

        let mut y3 = fu.frobenius_to();
        let fu2p = fu2.frobenius_to();
        let fu3p = fu3.frobenius_to();
        let y2 = fu2.frobenius_p2_to();

        let mut y0 = &fp * &fp2;
        y0.mul_ref(&fp3);

        let y1 = t1.conjugate_to();
        let y5 = fu2.conjugate_to();
        y3.conjugate();
        let mut y4 = &fu * &fu2p;
        y4.conjugate();

        let mut y6 = Self::mul_to(&fu3, &fu3p);
        y6.conjugate();

        let mut t0: Gfp12 = unsafe { MaybeUninit::uninit().assume_init() };
        y6.square_to_mut(&mut t0);
        t0.mul_ref(&y4).mul_ref(&y5);
        t1.set(&y3).mul_ref(&y5).mul_ref(&t0);
        t0.mul_ref(&y2);
        t1.square().mul_ref(&t0).square();
        t0.set(&t1).mul_ref(&y1);
        t1.mul_ref(&y0);
        t0.square().mul_ref(&t1);

        *self = t0;
        self
    }
}

gfp_ops_impl!(Gfp12);
