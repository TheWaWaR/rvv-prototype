use super::{constants::*, gfp::Gfp, gfp2::Gfp2, macros::gfp_ops_impl};
use core::mem::MaybeUninit;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gfp6(pub [Gfp2; 3]);

impl Gfp6 {
    pub fn x(&self) -> &Gfp2 {
        &self.0[0]
    }

    pub fn y(&self) -> &Gfp2 {
        &self.0[1]
    }

    pub fn z(&self) -> &Gfp2 {
        &self.0[2]
    }

    pub fn x_mut(&mut self) -> &mut Gfp2 {
        &mut self.0[0]
    }

    pub fn y_mut(&mut self) -> &mut Gfp2 {
        &mut self.0[1]
    }

    pub fn z_mut(&mut self) -> &mut Gfp2 {
        &mut self.0[2]
    }

    pub fn set_x(&mut self, x: &Gfp2) -> &mut Self {
        self.0[0].set(x);
        self
    }

    pub fn set_y(&mut self, y: &Gfp2) -> &mut Self {
        self.0[1].set(y);
        self
    }

    pub fn set_z(&mut self, z: &Gfp2) -> &mut Self {
        self.0[2].set(z);
        self
    }

    pub fn set(&mut self, other: &Gfp6) -> &mut Self {
        self.set_x(other.x());
        self.set_y(other.y());
        self.set_z(other.z());
        self
    }

    pub fn set_zero(&mut self) -> &mut Self {
        self.x_mut().set_zero();
        self.y_mut().set_zero();
        self.z_mut().set_zero();
        self
    }

    pub fn set_one(&mut self) -> &mut Self {
        self.x_mut().set_zero();
        self.y_mut().set_zero();
        self.z_mut().set_one();
        self
    }

    pub fn is_zero(&self) -> bool {
        self.x().is_zero() && self.y().is_zero() && self.z().is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.x().is_zero() && self.y().is_zero() && self.z().is_one()
    }

    pub fn neg_ref(&mut self) -> &mut Self {
        self.x_mut().neg_ref();
        self.y_mut().neg_ref();
        self.z_mut().neg_ref();
        self
    }

    pub fn neg_to(&self) -> Self {
        Self([self.x().neg_to(), self.y().neg_to(), self.x().neg_to()])
    }

    pub fn frobenius(&mut self) -> &mut Self {
        self.x_mut().conjugate();
        self.y_mut().conjugate();
        self.z_mut().conjugate();

        self.0[0] *= &constant_to_gfp2(&XI_TO_2P_MINUS2_OVER3);
        self.0[1] *= &constant_to_gfp2(&XI_TO_P_MINUS1_OVER3);
        self
    }

    pub fn frobenius_p2(&mut self) -> &mut Self {
        self.x_mut().mul_scalar(&Gfp(XI_TO_2P_SQUARED_MINUS2_OVER3));
        self.y_mut().mul_scalar(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER3));
        self
    }

    pub fn frobenius_p4(&mut self) -> &mut Self {
        self.x_mut().mul_scalar(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER3));
        self.y_mut().mul_scalar(&Gfp(XI_TO_2P_SQUARED_MINUS2_OVER3));
        self
    }

    pub fn add_ref(&mut self, b: &Gfp6) -> &mut Self {
        self.0[0] += b.x();
        self.0[1] += b.y();
        self.0[2] += b.z();
        self
    }

    pub fn add_to(a: &Gfp6, b: &Gfp6) -> Self {
        let mut r: Gfp6 = unsafe { MaybeUninit::uninit().assume_init() };
        r.0[0] = a.x() + b.x();
        r.0[1] = a.y() + b.y();
        r.0[2] = a.z() + b.z();
        r
    }

    pub fn sub_ref(&mut self, b: &Gfp6) -> &mut Self {
        self.0[0] -= b.x();
        self.0[1] -= b.y();
        self.0[2] -= b.z();
        self
    }

    pub fn sub_to(a: &Gfp6, b: &Gfp6) -> Self {
        let mut r: Gfp6 = unsafe { MaybeUninit::uninit().assume_init() };
        r.0[0] = a.x() - b.x();
        r.0[1] = a.y() - b.y();
        r.0[2] = a.z() - b.z();
        r
    }

    pub fn mul_ref(&mut self, b: &Gfp6) -> &mut Self {
        let v0 = self.z() * b.z();
        let v1 = self.y() * b.y();
        let v2 = self.x() * b.x();

        let mut t0 = self.x() + self.y();
        let mut t1 = b.x() + b.y();
        let mut tz = (&t0) * (&t1);
        tz.sub_ref(&v1).sub_ref(&v2).mul_xi().add_ref(&v0);

        t0.set(self.y());
        t0 += self.z();
        t1.set(b.y());
        t1 += b.z();
        let mut ty = (&t0) * (&t1);
        t0.set(&v2).mul_xi();
        ty.sub_ref(&v0).sub_ref(&v1).add_ref(&t0);

        t0.set(self.x());
        t0 += self.z();
        t1.set(b.x());
        t1 += b.z();
        let mut tx = (&t0) * (&t1);
        tx.sub_ref(&v0).add_ref(&v1).sub_ref(&v2);

        self.set_x(&tx);
        self.set_y(&ty);
        self.set_z(&tz);
        self
    }

    pub fn mul_to(a: &Gfp6, b: &Gfp6) -> Self {
        let v0 = a.z() * b.z();
        let v1 = a.y() * b.y();
        let v2 = a.x() * b.x();

        let mut t0 = a.x() + a.y();
        let mut t1 = b.x() + b.y();
        let mut tz = (&t0) * (&t1);
        tz.sub_ref(&v1).sub_ref(&v2).mul_xi().add_ref(&v0);

        t0.set(a.y());
        t0 += a.z();
        t1.set(b.y());
        t1 += b.z();
        let mut ty = (&t0) * (&t1);
        t0.set(&v2).mul_xi();
        ty.sub_ref(&v0).sub_ref(&v1).add_ref(&t0);

        t0.set(a.x());
        t0 += a.z();
        t1.set(b.x());
        t1 += b.z();
        let mut tx = (&t0) * (&t1);
        tx.sub_ref(&v0).add_ref(&v1).sub_ref(&v2);

        let mut r: Gfp6 = unsafe { MaybeUninit::uninit().assume_init() };
        r.set_x(&tx);
        r.set_y(&ty);
        r.set_z(&tz);
        r
    }

    pub fn mul_scalar(&mut self, b: &Gfp2) -> &mut Self {
        self.x_mut().mul_ref(b);
        self.y_mut().mul_ref(b);
        self.z_mut().mul_ref(b);
        self
    }

    pub fn mul_gfp(&mut self, b: &Gfp) -> &mut Self {
        self.x_mut().mul_scalar(b);
        self.y_mut().mul_scalar(b);
        self.z_mut().mul_scalar(b);
        self
    }

    // This is actually slower when converting to the mul_tau_to version
    pub fn mul_tau(&mut self) -> &mut Self {
        let tz = self.x().mul_xi_to();

        self.0[0].set(&self.y().clone());
        self.0[1].set(&self.z().clone());
        self.0[2].set(&tz);
        self
    }

    pub fn square(&mut self) -> &mut Self {
        *self = self.square_to();
        self
    }

    pub fn square_to(&self) -> Self {
        let v0 = self.z().square_to();
        let v1 = self.y().square_to();
        let v2 = self.x().square_to();

        let mut c0 = self.x() + self.y();
        c0.square().sub_ref(&v1).sub_ref(&v2).mul_xi().add_ref(&v0);

        let mut c1 = self.y() + self.z();
        c1.square().sub_ref(&v0).sub_ref(&v1);
        let xi_v2 = v2.mul_xi_to();
        c1.add_ref(&xi_v2);

        let mut c2 = self.x() + self.z();
        c2.square().sub_ref(&v0).add_ref(&v1).sub_ref(&v2);

        Gfp6([c2, c1, c0])
    }

    pub fn invert(&mut self) -> &mut Self {
        let mut t1 = self.x() * self.y();
        t1.mul_xi();

        let mut a = self.z().square_to();
        a.sub_ref(&t1);

        let mut b = self.x().square_to();
        b.mul_xi();
        t1 = self.y() * self.z();
        b.sub_ref(&t1);

        let mut c = self.y().square_to();
        t1 = self.x() * self.z();
        c.sub_ref(&t1);

        let mut f = (&c) * self.y();
        f.mul_xi();
        t1 = (&a) * self.z();
        f.add_ref(&t1);
        t1 = (&b) * self.x();
        t1.mul_xi();
        f.add_ref(&t1);

        f.invert();

        self.set_x(&((&c) * (&f)));
        self.set_y(&((&b) * (&f)));
        self.set_z(&((&a) * (&f)));

        self
    }
}

gfp_ops_impl!(Gfp6);
