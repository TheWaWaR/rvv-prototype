use super::{
    casts::*,
    constants::*,
    gfp::{self, Gfp},
    gfp2::Gfp2,
    macros::gfp_ops_impl,
};
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
        let dst = gfp2_to_gfp_slice_mut(&mut self.0);
        gfp::neg(dst);
        self
    }

    pub fn neg_to(&self) -> Self {
        let mut r: Gfp6 = unsafe { MaybeUninit::uninit().assume_init() };

        let dst = gfp2_to_gfp_slice_mut(&mut r.0);
        let src = gfp2_to_gfp_slice(&self.0);
        gfp::neg_to(src, dst);

        r
    }

    pub fn frobenius(&mut self) -> &mut Self {
        self.x_mut().conjugate();
        self.y_mut().conjugate();
        self.z_mut().conjugate();

        self.0[0] *= &constant_to_gfp2(&XI_TO_2P_MINUS2_OVER3);
        self.0[1] *= &constant_to_gfp2(&XI_TO_P_MINUS1_OVER3);
        self
    }

    pub fn frobenius_to(&self) -> Self {
        let mut r = Gfp6([
            self.x().conjugate_to(),
            self.y().conjugate_to(),
            self.z().conjugate_to(),
        ]);
        r.x_mut().mul_ref(&constant_to_gfp2(&XI_TO_2P_MINUS2_OVER3));
        r.y_mut().mul_ref(&constant_to_gfp2(&XI_TO_P_MINUS1_OVER3));
        r
    }

    pub fn frobenius_p2(&mut self) -> &mut Self {
        self.x_mut().mul_scalar(&Gfp(XI_TO_2P_SQUARED_MINUS2_OVER3));
        self.y_mut().mul_scalar(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER3));
        self
    }

    pub fn frobenius_p2_to(&self) -> Self {
        Self([
            self.x().mul_scalar_to(&Gfp(XI_TO_2P_SQUARED_MINUS2_OVER3)),
            self.y().mul_scalar_to(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER3)),
            self.z().clone(),
        ])
    }

    pub fn frobenius_p4(&mut self) -> &mut Self {
        self.x_mut().mul_scalar(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER3));
        self.y_mut().mul_scalar(&Gfp(XI_TO_2P_SQUARED_MINUS2_OVER3));
        self
    }

    pub fn double(&mut self) -> &mut Self {
        let dst = gfp2_to_gfp_slice_mut(&mut self.0);

        gfp::double(dst);
        self
    }

    pub fn add_ref(&mut self, b: &Gfp6) -> &mut Self {
        let dst = gfp2_to_gfp_slice_mut(&mut self.0);
        let src = gfp2_to_gfp_slice(&b.0);

        gfp::add_mov(dst, src);
        self
    }

    #[inline(always)]
    pub fn add_to(a: &Gfp6, b: &Gfp6) -> Self {
        let mut r: Gfp6 = unsafe { MaybeUninit::uninit().assume_init() };
        Self::add_to_mut(a, b, &mut r);
        r
    }

    pub fn add_to_mut(a: &Gfp6, b: &Gfp6, c: &mut Gfp6) {
        let c = gfp2_to_gfp_slice_mut(&mut c.0);
        let a = gfp2_to_gfp_slice(&a.0);
        let b = gfp2_to_gfp_slice(&b.0);
        gfp::add(a, b, c);
    }

    pub fn sub_ref(&mut self, b: &Gfp6) -> &mut Self {
        let dst = gfp2_to_gfp_slice_mut(&mut self.0);
        let src = gfp2_to_gfp_slice(&b.0);

        gfp::sub_mov(dst, src);
        self
    }

    pub fn sub_to(a: &Gfp6, b: &Gfp6) -> Self {
        let mut r: Gfp6 = unsafe { MaybeUninit::uninit().assume_init() };

        let c = gfp2_to_gfp_slice_mut(&mut r.0);
        let a = gfp2_to_gfp_slice(&a.0);
        let b = gfp2_to_gfp_slice(&b.0);
        gfp::sub(a, b, c);

        r
    }

    pub fn mul_ref(&mut self, b: &Gfp6) -> &mut Self {
        let v0 = self.z() * b.z();
        let v1 = self.y() * b.y();
        let v2 = self.x() * b.x();

        let z = self.z().clone();
        *self.z_mut() = self.x() + self.y();
        self.y_mut().add_ref(&z);
        self.x_mut().add_ref(&z);

        let t1_z = b.x() + b.y();
        let t1_y = b.y() + b.z();
        let t1_x = b.x() + b.z();

        self.z_mut()
            .mul_ref(&t1_z)
            .sub_ref(&v1)
            .sub_ref(&v2)
            .mul_xi()
            .add_ref(&v0);
        self.y_mut()
            .mul_ref(&t1_y)
            .sub_ref(&v0)
            .sub_ref(&v1)
            .add_ref(&v2.mul_xi_to());
        self.x_mut()
            .mul_ref(&t1_x)
            .sub_ref(&v0)
            .add_ref(&v1)
            .sub_ref(&v2);

        self
    }

    pub fn mul_to(a: &Gfp6, b: &Gfp6) -> Self {
        let mut r: Gfp6 = unsafe { MaybeUninit::uninit().assume_init() };

        let v0 = a.z() * b.z();
        let v1 = a.y() * b.y();
        let v2 = a.x() * b.x();

        let z = a.z().clone();
        *r.z_mut() = a.x() + a.y();
        *r.y_mut() = a.y() + &z;
        *r.x_mut() = a.x() + &z;

        let t1_z = b.x() + b.y();
        let t1_y = b.y() + b.z();
        let t1_x = b.x() + b.z();

        r.z_mut()
            .mul_ref(&t1_z)
            .sub_ref(&v1)
            .sub_ref(&v2)
            .mul_xi()
            .add_ref(&v0);
        r.y_mut()
            .mul_ref(&t1_y)
            .sub_ref(&v0)
            .sub_ref(&v1)
            .add_ref(&v2.mul_xi_to());
        r.x_mut()
            .mul_ref(&t1_x)
            .sub_ref(&v0)
            .add_ref(&v1)
            .sub_ref(&v2);
        r
    }

    pub fn mul_scalar(&mut self, b: &Gfp2) -> &mut Self {
        self.x_mut().mul_ref(b);
        self.y_mut().mul_ref(b);
        self.z_mut().mul_ref(b);
        self
    }

    pub fn mul_scalar_to(&self, b: &Gfp2) -> Self {
        Self([self.x() * b, self.y() * b, self.z() * b])
    }

    pub fn mul_gfp(&mut self, b: &Gfp) -> &mut Self {
        let dst = gfp2_to_gfp_slice_mut(&mut self.0);

        gfp::mul_mov_scalar(dst, b);

        self
    }

    pub fn mul_gfp_to(&self, b: &Gfp) -> Self {
        let mut r: Gfp6 = unsafe { MaybeUninit::uninit().assume_init() };

        let c = gfp2_to_gfp_slice_mut(&mut r.0);
        let a = gfp2_to_gfp_slice(&self.0);
        gfp::mul_scalar(a, b, c);

        r
    }

    pub fn mul_tau(&mut self) -> &mut Self {
        let tz = self.x().mul_xi_to();

        self.0[0].set(&self.y().clone());
        self.0[1].set(&self.z().clone());
        self.0[2].set(&tz);
        self
    }

    pub fn mul_tau_to(&self) -> Self {
        Gfp6([self.y().clone(), self.z().clone(), self.x().mul_xi_to()])
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

    pub fn invert_to(self) -> Self {
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

        Gfp6([&c * &f, &b * &f, &a * &f])
    }
}

gfp_ops_impl!(Gfp6);
