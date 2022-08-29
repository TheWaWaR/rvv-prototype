use super::{
    constants::*, curve::CurvePoint, gfp::Gfp, gfp12::Gfp12, gfp2::Gfp2, gfp6::Gfp6,
    twist::TwistPoint,
};

const SIX_U_PLUS2_NAF: &[i8] = &[
    0, 0, 0, 1, 0, 1, 0, -1, 0, 0, 1, -1, 0, 0, 1, 0, 0, 1, 1, 0, -1, 0, 0, 1, 0, -1, 0, 0, 0, 0,
    1, 1, 1, 0, 0, -1, 0, 0, 1, 0, 0, 0, 0, 0, -1, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 1, 1, 0, -1, 0,
    0, 1, 0, 1, 1,
];

pub fn miller(mut a_affine: TwistPoint, mut b_affine: CurvePoint) -> Gfp12 {
    let mut ret = Gfp12::one();

    a_affine.make_affine();
    b_affine.make_affine();

    let mut minus_a = a_affine.clone();
    minus_a.neg_ref();

    let mut r = a_affine.clone();

    let mut r2 = a_affine.y().square_to();

    for i in (1..SIX_U_PLUS2_NAF.len()).rev() {
        let (mut a, mut b, mut c, mut new_r) = line_function_double(&r, &b_affine);
        if i != SIX_U_PLUS2_NAF.len() - 1 {
            ret.square();
        }

        mul_line(&a, &b, &c, &mut ret);
        r = new_r;

        match SIX_U_PLUS2_NAF[i - 1] {
            1 => {
                // https://github.com/rust-lang/rust/issues/71126
                let results = line_function_add(&r, &a_affine, &b_affine, &r2);
                a = results.0;
                b = results.1;
                c = results.2;
                new_r = results.3;
            }
            -1 => {
                let results = line_function_add(&r, &minus_a, &b_affine, &r2);
                a = results.0;
                b = results.1;
                c = results.2;
                new_r = results.3;
            }
            _ => continue,
        }

        mul_line(&a, &b, &c, &mut ret);
        r = new_r;
    }

    let mut q1 = TwistPoint::default();
    q1.x_mut()
        .set(a_affine.x())
        .conjugate()
        .mul_ref(&constant_to_gfp2(&XI_TO_P_MINUS1_OVER3));
    q1.y_mut()
        .set(a_affine.y())
        .conjugate()
        .mul_ref(&constant_to_gfp2(&XI_TO_P_MINUS1_OVER2));
    q1.z_mut().set_one();
    q1.t_mut().set_one();

    let mut minus_q2 = TwistPoint::default();
    minus_q2
        .x_mut()
        .set(a_affine.x())
        .mul_scalar(&Gfp(XI_TO_P_SQUARED_MINUS1_OVER3));
    minus_q2.y_mut().set(a_affine.y());
    minus_q2.z_mut().set_one();
    minus_q2.t_mut().set_one();

    q1.y().square_to_mut(&mut r2);
    let (a, b, c, new_r) = line_function_add(&r, &q1, &b_affine, &r2);
    mul_line(&a, &b, &c, &mut ret);
    r = new_r;

    minus_q2.y().square_to_mut(&mut r2);
    let (a, b, c, _new_r) = line_function_add(&r, &minus_q2, &b_affine, &r2);
    mul_line(&a, &b, &c, &mut ret);
    // r = new_r;

    ret
}

pub(crate) fn line_function_add(
    r: &TwistPoint,
    p: &TwistPoint,
    q: &CurvePoint,
    r2: &Gfp2,
) -> (Gfp2, Gfp2, Gfp2, TwistPoint) {
    let bb = p.x() * r.t();

    let mut dd = p.y() + r.z();
    dd.square().sub_ref(r2).sub_ref(r.t()).mul_ref(r.t());

    let hh = &bb - r.x();
    let ii = hh.square_to();

    let mut ee = &ii + &ii;
    ee = &ee + &ee;

    let jj = &hh * &ee;

    let mut l1 = &dd - r.y();
    l1.sub_ref(r.y());

    let vv = r.x() * &ee;

    let mut r_out = TwistPoint::default();
    r_out
        .x_mut()
        .set(&l1)
        .square()
        .sub_ref(&jj)
        .sub_ref(&vv)
        .sub_ref(&vv);

    r_out
        .z_mut()
        .set(r.z())
        .add_ref(&hh)
        .square()
        .sub_ref(r.t())
        .sub_ref(&ii);

    let mut t = &vv - r_out.x();
    t.mul_ref(&l1);
    let mut t2 = r.y() * &jj;
    t2 = &t2 + &t2;
    r_out.y_mut().set(&t).sub_ref(&t2);

    let r_out_z_squared = r_out.z().square_to();
    r_out.t_mut().set(&r_out_z_squared);

    t.set(p.y())
        .add_ref(r_out.z())
        .square()
        .sub_ref(&r2)
        .sub_ref(r_out.t());

    t2 = &l1 * p.x();
    t2 = &t2 + &t2;
    let a = &t2 - &t;

    let mut c = r_out.z().mul_scalar_to(q.y());
    c = &c + &c;

    let mut b = -l1;
    b.mul_scalar(q.x());
    b = &b + &b;

    (a, b, c, r_out)
}

pub(crate) fn line_function_double(
    r: &TwistPoint,
    q: &CurvePoint,
) -> (Gfp2, Gfp2, Gfp2, TwistPoint) {
    let aa = r.x().square_to();
    let bb = r.y().square_to();
    let cc = bb.square_to();

    let mut dd = r.x() + &bb;
    dd.square().sub_ref(&aa).sub_ref(&cc);
    dd = &dd + &dd;

    let mut ee = &aa + &aa;
    ee.add_ref(&aa);

    let gg = ee.square_to();

    let mut r_out = TwistPoint::default();
    r_out.x_mut().set(&gg).sub_ref(&dd).sub_ref(&dd);

    r_out
        .z_mut()
        .set(r.y())
        .add_ref(r.z())
        .square()
        .sub_ref(&bb)
        .sub_ref(r.t());

    *r_out.y_mut() = &dd - r_out.x();
    r_out.y_mut().mul_ref(&ee);
    let mut t = &cc + &cc;
    t = &t + &t;
    t = &t + &t;
    r_out.y_mut().sub_ref(&t);

    let r_out_z_squared = r_out.z().square_to();
    r_out.t_mut().set(&r_out_z_squared);

    t = &ee * r.t();
    t = &t + &t;
    let mut b = -t;
    b.mul_scalar(q.x());

    let mut a = r.x() + &ee;
    a.square().sub_ref(&aa).sub_ref(&gg);
    t = &bb + &bb;
    t = &t + &t;
    a.sub_ref(&t);

    let mut c = r_out.z() * r.t();
    c = &c + &c;
    c.mul_scalar(q.y());

    (a, b, c, r_out)
}

pub(crate) fn mul_line(a: &Gfp2, b: &Gfp2, c: &Gfp2, ret: &mut Gfp12) {
    let mut a2 = Gfp6::default();
    a2.set_y(a);
    a2.set_z(b);
    a2.mul_ref(ret.x());
    let t3 = ret.y().mul_scalar_to(c);

    let t = b + c;
    let mut t2 = Gfp6::default();
    t2.set_y(a);
    t2.set_z(&t);
    *ret.x_mut() = ret.x() + ret.y();

    ret.set_y(&t3);

    ret.x_mut().mul_ref(&t2).sub_ref(&a2).sub_ref(&t3);
    a2.mul_tau();
    ret.y_mut().add_ref(&a2);
}
