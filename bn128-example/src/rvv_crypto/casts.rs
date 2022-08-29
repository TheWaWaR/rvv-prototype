use super::{gfp::Gfp, gfp2::Gfp2, gfp6::Gfp6};

use core::ptr;

pub fn gfp2_to_gfp_slice(s: &[Gfp2]) -> &[Gfp] {
    let len = s.len() * 2;
    let p = s.as_ptr() as *mut Gfp;
    let p = ptr::slice_from_raw_parts(p, len);
    unsafe { &*p }
}

pub fn gfp2_to_gfp_slice_mut(s: &mut [Gfp2]) -> &mut [Gfp] {
    let len = s.len() * 2;
    let p = s.as_mut_ptr() as *mut Gfp;
    let p = ptr::slice_from_raw_parts_mut(p, len);
    unsafe { &mut *p }
}

pub fn gfp6_to_gfp_slice(s: &[Gfp6]) -> &[Gfp] {
    let len = s.len() * 6;
    let p = s.as_ptr() as *mut Gfp;
    let p = ptr::slice_from_raw_parts(p, len);
    unsafe { &*p }
}

pub fn gfp6_to_gfp_slice_mut(s: &mut [Gfp6]) -> &mut [Gfp] {
    let len = s.len() * 6;
    let p = s.as_mut_ptr() as *mut Gfp;
    let p = ptr::slice_from_raw_parts_mut(p, len);
    unsafe { &mut *p }
}
