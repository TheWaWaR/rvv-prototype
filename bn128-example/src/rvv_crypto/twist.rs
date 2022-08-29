use super::{
    gfp::{self, Gfp},
    gfp2::Gfp2,
    Error,
};
use core::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TwistPoint(pub [Gfp2; 4]);

impl TryFrom<&[u8]> for TwistPoint {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut xx_xy_yx_yy: [Gfp; 4] = [
            value[0..32].try_into()?,
            value[32..64].try_into()?,
            value[64..96].try_into()?,
            value[96..128].try_into()?,
        ];
        gfp::mont_encode(&mut xx_xy_yx_yy);
        let [xx, xy, yx, yy] = xx_xy_yx_yy;
        let mut point = TwistPoint([
            Gfp2([xx, xy]),
            Gfp2([yx, yy]),
            Gfp2::default(),
            Gfp2::default(),
        ]);

        if point.x().is_zero() && point.y().is_zero() {
            point.y_mut().set_one();
            point.z_mut().set_zero();
            point.t_mut().set_zero();
        } else {
            point.z_mut().set_one();
            point.t_mut().set_one();

            if !point.is_on_curve() {
                return Err("malformed twist point!".into());
            }
        }
        Ok(point)
    }
}

pub const TWIST_B: Gfp2 = Gfp2([
    Gfp([
        0x38e7ecccd1dcff67,
        0x65f0b37d93ce0d3e,
        0xd749d0dd22ac00aa,
        0x0141b9ce4a688d4d,
    ]),
    Gfp([
        0x3bf938e377b802a8,
        0x020b1b273633535d,
        0x26b7edf049755260,
        0x2514c6324384a86d,
    ]),
]);

pub const TWIST_GEN: TwistPoint = TwistPoint([
    Gfp2([
        Gfp([
            0xafb4737da84c6140,
            0x6043dd5a5802d8c4,
            0x09e950fc52a02f86,
            0x14fef0833aea7b6b,
        ]),
        Gfp([
            0x8e83b5d102bc2026,
            0xdceb1935497b0172,
            0xfbb8264797811adf,
            0x19573841af96503b,
        ]),
    ]),
    Gfp2([
        Gfp([
            0x64095b56c71856ee,
            0xdc57f922327d3cbb,
            0x55f935be33351076,
            0x0da4a0e693fd6482,
        ]),
        Gfp([
            0x619dfa9d886be9f6,
            0xfe7fd297f59e9b78,
            0xff9e1a62231b7dfe,
            0x28fd7eebae9e4206,
        ]),
    ]),
    Gfp2([
        // Gfp::new_from_int64(0)
        Gfp([0, 0, 0, 0]),
        // Gfp::new_from_int64(1)
        Gfp([
            15230403791020821917,
            754611498739239741,
            7381016538464732716,
            1011752739694698287,
        ]),
    ]),
    Gfp2([
        // Gfp::new_from_int64(0)
        Gfp([0, 0, 0, 0]),
        // Gfp::new_from_int64(1)
        Gfp([
            15230403791020821917,
            754611498739239741,
            7381016538464732716,
            1011752739694698287,
        ]),
    ]),
]);

impl TwistPoint {
    pub fn x(&self) -> &Gfp2 {
        &self.0[0]
    }

    pub fn y(&self) -> &Gfp2 {
        &self.0[1]
    }

    pub fn z(&self) -> &Gfp2 {
        &self.0[2]
    }

    pub fn t(&self) -> &Gfp2 {
        &self.0[3]
    }

    pub fn x_slice(&self) -> &[Gfp2] {
        &self.0[0..1]
    }

    pub fn y_slice(&self) -> &[Gfp2] {
        &self.0[1..2]
    }

    pub fn z_slice(&self) -> &[Gfp2] {
        &self.0[2..3]
    }

    pub fn t_slice(&self) -> &[Gfp2] {
        &self.0[3..4]
    }

    pub fn x_slice_mut(&mut self) -> &mut [Gfp2] {
        &mut self.0[0..1]
    }

    pub fn y_slice_mut(&mut self) -> &mut [Gfp2] {
        &mut self.0[1..2]
    }

    pub fn z_slice_mut(&mut self) -> &mut [Gfp2] {
        &mut self.0[2..3]
    }

    pub fn t_slice_mut(&mut self) -> &mut [Gfp2] {
        &mut self.0[3..4]
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

    pub fn t_mut(&mut self) -> &mut Gfp2 {
        &mut self.0[3]
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

    pub fn set_t(&mut self, t: &Gfp2) -> &mut Self {
        self.0[3].set(t);
        self
    }

    pub fn set(&mut self, other: &TwistPoint) -> &mut Self {
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

        let y2 = self.y().square_to();
        let mut x3 = self.x().square_to();
        x3.mul_ref(self.x()).add_ref(&TWIST_B);

        y2 == x3
    }

    pub fn set_infinity(&mut self) -> &mut Self {
        self.x_mut().set_zero();
        self.y_mut().set_one();
        self.z_mut().set_zero();
        self.t_mut().set_zero();
        self
    }

    pub fn is_infinity(&self) -> bool {
        self.z().is_zero()
    }

    pub fn make_affine(&mut self) -> &mut Self {
        if self.z().is_one() {
            return self;
        } else if self.z().is_zero() {
            self.x_mut().set_zero();
            self.y_mut().set_one();
            self.t_mut().set_zero();
            return self;
        }

        let mut z_inv = self.z().clone();
        z_inv.invert();
        let mut t = self.y() * &z_inv;
        let z_inv2 = z_inv.square_to();
        self.y_mut().set(&t).mul_ref(&z_inv2);
        t.set(self.x()).mul_ref(&z_inv2);
        self.x_mut().set(&t);
        self.z_mut().set_one();
        self.t_mut().set_one();
        self
    }

    pub fn neg_ref(&mut self) -> &mut Self {
        self.y_mut().neg_ref();
        self.t_mut().set_zero();
        self
    }

    pub fn neg_to(&self) -> Self {
        Self([
            self.x().clone(),
            self.y().neg_to(),
            self.z().clone(),
            Gfp2::zero(),
        ])
    }
}

pub fn double(a: &TwistPoint, c: &mut TwistPoint) {
    let aa = a.x().square_to();
    let bb = a.y().square_to();
    let cc = bb.square_to();

    let mut t = a.x() + &bb;
    let mut t2 = t.square_to();
    t.set(&t2).sub_ref(&aa);
    t2.set(&t).sub_ref(&cc);
    let d = (&t2) + (&t2);
    t = (&aa) + (&aa);
    let e = (&t) + (&aa);
    let f = e.square_to();

    t = (&d) + (&d);
    c.x_mut().set(&f).sub_ref(&t);

    let cz = a.y() * a.z();
    c.z_mut().set(&cz).add_ref(&cz);

    t = (&cc) + (&cc);
    t2 = (&t) + (&t);
    t = (&t2) + (&t2);
    let cy = (&d) - c.x();
    t2 = (&e) * (&cy);
    c.y_mut().set(&t2).sub_ref(&t);
}

pub fn add(a: &TwistPoint, b: &TwistPoint, c: &mut TwistPoint) {
    if a.is_infinity() {
        c.set(b);
        return;
    }
    if b.is_infinity() {
        c.set(a);
        return;
    }

    let z12 = a.z().square_to();
    let z22 = b.z().square_to();
    let u1 = a.x() * &z22;
    let u2 = b.x() * &z12;

    let mut t = b.z() * &z22;
    let s1 = a.y() * &t;

    t = a.z() * &z12;
    let s2 = b.y() * &t;

    let h = &u2 - &u1;
    let x_equal = h.is_zero();

    t = &h + &h;
    let i = t.square_to();
    let j = &h * &i;

    t = &s2 - &s1;
    let y_equal = t.is_zero();
    if x_equal && y_equal {
        double(a, c);
        return;
    }
    let r = &t + &t;

    let v = &u1 * &i;

    let mut t4 = r.square_to();
    t = &v + &v;
    let mut t6 = &t4 - &j;
    c.x_mut().set(&t6).sub_ref(&t);

    t = &v - c.x();
    t4 = &s1 * &j;
    t6 = &t4 + &t4;
    t4 = &r * &t;
    c.y_mut().set(&t4).sub_ref(&t6);

    t = a.z() + b.z();
    t4 = t.square_to();
    t = &t4 - &z12;
    t4 = t - &z22;
    c.z_mut().set(&t4).mul_ref(&h);
}
