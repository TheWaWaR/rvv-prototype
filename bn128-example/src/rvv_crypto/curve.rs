use super::{
    gfp::{self, Gfp},
    Error,
};
use core::convert::{TryFrom, TryInto};
use core::mem::MaybeUninit;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurvePoint(pub [Gfp; 4]);

impl TryFrom<&[u8]> for CurvePoint {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut xy: [Gfp; 2] = [value[0..32].try_into()?, value[32..64].try_into()?];
        gfp::mont_encode(&mut xy);
        let [x, y] = xy;
        let mut point = CurvePoint([x, y, Gfp::default(), Gfp::default()]);

        if point.x().is_zero() && point.y().is_zero() {
            point.set_y(&gfp::ONE);
            point.set_z(&gfp::ZERO);
            point.set_t(&gfp::ZERO);
        } else {
            point.set_z(&gfp::ONE);
            point.set_t(&gfp::ONE);

            if !point.is_on_curve() {
                return Err("malformed curve point!".into());
            }
        }
        Ok(point)
    }
}

// Gfp::new_from_int64(3)
pub const CURVE_B: [Gfp; 1] = [Gfp([
    8797723225643362519,
    2263834496217719225,
    3696305541684646532,
    3035258219084094862,
])];

pub const CURVE_GEN: CurvePoint = CurvePoint([
    // x: Gfp::new_from_int64(1)
    Gfp([
        15230403791020821917,
        754611498739239741,
        7381016538464732716,
        1011752739694698287,
    ]),
    // y: Gfp::new_from_int64(2)
    Gfp([
        12014063508332092218,
        1509222997478479483,
        14762033076929465432,
        2023505479389396574,
    ]),
    // z: Gfp::new_from_int64(1)
    Gfp([
        15230403791020821917,
        754611498739239741,
        7381016538464732716,
        1011752739694698287,
    ]),
    // t: Gfp::new_from_int64(1)
    Gfp([
        15230403791020821917,
        754611498739239741,
        7381016538464732716,
        1011752739694698287,
    ]),
]);

impl CurvePoint {
    pub fn x(&self) -> &Gfp {
        &self.0[0]
    }

    pub fn y(&self) -> &Gfp {
        &self.0[1]
    }

    pub fn z(&self) -> &Gfp {
        &self.0[2]
    }

    pub fn t(&self) -> &Gfp {
        &self.0[3]
    }

    pub fn x_slice(&self) -> &[Gfp] {
        &self.0[0..1]
    }

    pub fn y_slice(&self) -> &[Gfp] {
        &self.0[1..2]
    }

    pub fn z_slice(&self) -> &[Gfp] {
        &self.0[2..3]
    }

    pub fn t_slice(&self) -> &[Gfp] {
        &self.0[3..4]
    }

    pub fn xy_slice(&self) -> &[Gfp] {
        &self.0[0..2]
    }

    pub fn x_slice_mut(&mut self) -> &mut [Gfp] {
        &mut self.0[0..1]
    }

    pub fn y_slice_mut(&mut self) -> &mut [Gfp] {
        &mut self.0[1..2]
    }

    pub fn z_slice_mut(&mut self) -> &mut [Gfp] {
        &mut self.0[2..3]
    }

    pub fn t_slice_mut(&mut self) -> &mut [Gfp] {
        &mut self.0[3..4]
    }

    pub fn x_mut(&mut self) -> &mut Gfp {
        &mut self.0[0]
    }

    pub fn y_mut(&mut self) -> &mut Gfp {
        &mut self.0[1]
    }

    pub fn z_mut(&mut self) -> &mut Gfp {
        &mut self.0[2]
    }

    pub fn t_mut(&mut self) -> &mut Gfp {
        &mut self.0[3]
    }

    pub fn set_x(&mut self, x: &Gfp) -> &mut Self {
        self.0[0].set(x);
        self
    }

    pub fn set_y(&mut self, y: &Gfp) -> &mut Self {
        self.0[1].set(y);
        self
    }

    pub fn set_z(&mut self, z: &Gfp) -> &mut Self {
        self.0[2].set(z);
        self
    }

    pub fn set_t(&mut self, t: &Gfp) -> &mut Self {
        self.0[3].set(t);
        self
    }

    pub fn set(&mut self, other: &CurvePoint) -> &mut Self {
        self.set_x(other.x());
        self.set_y(other.y());
        self.set_z(other.z());
        self.set_t(other.t());
        self
    }

    pub fn is_on_curve(&mut self) -> bool {
        self.make_affine();
        if self.is_infinity() {
            return true;
        }

        let mut y2: [Gfp; 1] = unsafe { MaybeUninit::uninit().assume_init() };
        gfp::mul(self.y_slice(), self.y_slice(), &mut y2);
        let mut x3: [Gfp; 1] = unsafe { MaybeUninit::uninit().assume_init() };
        gfp::mul(self.x_slice(), self.x_slice(), &mut x3);
        gfp::mul_mov(&mut x3, self.x_slice());
        gfp::add_mov(&mut x3, &CURVE_B);

        y2[0] == x3[0]
    }

    pub fn set_infinity(&mut self) -> &mut Self {
        self.x_mut().set(&gfp::ZERO);
        self.y_mut().set(&gfp::ONE);
        self.z_mut().set(&gfp::ZERO);
        self.t_mut().set(&gfp::ZERO);
        self
    }

    pub fn is_infinity(&self) -> bool {
        self.z() == &gfp::ZERO
    }

    pub fn make_affine(&mut self) -> &mut Self {
        if self.z() == &gfp::ONE {
            return self;
        } else if self.z() == &gfp::ZERO {
            return self.set_x(&gfp::ZERO).set_y(&gfp::ONE).set_t(&gfp::ZERO);
        }

        let z_inv = [self.z().invert_to()];

        let mut t = [Gfp::default()];
        gfp::mul(self.y_slice(), &z_inv, &mut t);
        let mut z_inv2 = [Gfp::default()];
        gfp::mul(&z_inv, &z_inv, &mut z_inv2);

        gfp::mul_mov(self.x_slice_mut(), &z_inv2);
        gfp::mul(&t, &z_inv2, self.y_slice_mut());

        self.set_z(&gfp::ONE).set_t(&gfp::ONE)
    }

    pub fn neg_ref(&mut self) -> &mut Self {
        gfp::neg(self.y_slice_mut());
        self.set_t(&gfp::ZERO)
    }
}

pub fn double(a: &CurvePoint, c: &mut CurvePoint) {
    let mut aa_bb = [Gfp::default(), Gfp::default()];
    gfp::mul(a.xy_slice(), a.xy_slice(), &mut aa_bb);
    let aa = &aa_bb[0..1];
    let bb = &aa_bb[1..2];
    let mut cc = [Gfp::default()];
    gfp::mul(bb, bb, &mut cc);

    let mut t = [Gfp::default()];
    gfp::add(a.x_slice(), bb, &mut t);
    let mut t2 = [Gfp::default()];
    gfp::mul(&t, &t, &mut t2);
    gfp::sub(&t2, aa, &mut t);
    gfp::sub(&t, &cc, &mut t2);

    let mut d = [Gfp::default()];
    gfp::add(&t2, &t2, &mut d);
    gfp::add(aa, aa, &mut t);
    let mut e = [Gfp::default()];
    gfp::add(&t, aa, &mut e);
    let mut f = [Gfp::default()];
    gfp::mul(&e, &e, &mut f);

    gfp::add(&d, &d, &mut t);
    gfp::sub(&f, &t, c.x_slice_mut());

    gfp::mul(a.y_slice(), a.z_slice(), c.z_slice_mut());
    gfp::double(c.z_slice_mut());

    gfp::add(&cc, &cc, &mut t);
    gfp::add(&t, &t, &mut t2);
    gfp::add(&t2, &t2, &mut t);
    let cx = [c.x().clone()];
    gfp::sub(&d, &cx, c.y_slice_mut());
    gfp::mul(&e, c.y_slice(), &mut t2);
    gfp::sub(&t2, &t, c.y_slice_mut());
}

pub fn add(a: &CurvePoint, b: &CurvePoint, c: &mut CurvePoint) {
    if a.is_infinity() {
        c.set(b);
        return;
    }
    if b.is_infinity() {
        c.set(a);
        return;
    }

    let mut z12 = [Gfp::default()];
    gfp::mul(a.z_slice(), a.z_slice(), &mut z12);
    let mut z22 = [Gfp::default()];
    gfp::mul(b.z_slice(), b.z_slice(), &mut z22);

    let mut u1 = [Gfp::default()];
    gfp::mul(a.x_slice(), &z22, &mut u1);
    let mut u2 = [Gfp::default()];
    gfp::mul(b.x_slice(), &z12, &mut u2);

    let mut t = [Gfp::default()];
    gfp::mul(b.z_slice(), &z22, &mut t);
    let mut s1 = [Gfp::default()];
    gfp::mul(a.y_slice(), &t, &mut s1);

    let mut s2 = [Gfp::default()];
    gfp::mul(a.z_slice(), &z12, &mut t);
    gfp::mul(b.y_slice(), &t, &mut s2);

    let mut h = [Gfp::default()];
    gfp::sub(&u2, &u1, &mut h);
    let x_equal = &h[0] == &gfp::ZERO;

    gfp::add(&h, &h, &mut t);
    let mut i = [Gfp::default()];
    gfp::mul(&t, &t, &mut i);
    let mut j = [Gfp::default()];
    gfp::mul(&h, &i, &mut j);

    gfp::sub(&s2, &s1, &mut t);
    let y_equal = &t[0] == &gfp::ZERO;
    if x_equal && y_equal {
        double(a, c);
        return;
    }

    let mut r = [Gfp::default()];
    gfp::add(&t, &t, &mut r);

    let mut v = [Gfp::default()];
    gfp::mul(&u1, &i, &mut v);

    let mut t4 = [Gfp::default()];
    gfp::mul(&r, &r, &mut t4);
    gfp::add(&v, &v, &mut t);
    let mut t6 = [Gfp::default()];
    gfp::sub(&t4, &j, &mut t6);

    gfp::sub(&t6, &t, c.x_slice_mut());

    gfp::sub(&v, c.x_slice(), &mut t);
    gfp::mul(&s1, &j, &mut t4);
    gfp::add(&t4, &t4, &mut t6);
    gfp::mul(&r, &t, &mut t4);
    gfp::sub(&t4, &t6, c.y_slice_mut());

    gfp::add(a.z_slice(), b.z_slice(), &mut t);
    gfp::mul(&t, &t, &mut t4);
    gfp::sub(&t4, &z12, &mut t);
    gfp::sub(&t, &z22, &mut t4);
    gfp::mul(&t4, &h, c.z_slice_mut());
}
