use super::constants::{NP, P2};

extern "C" {
    fn _zz_preload(np: *const u64, p2: *const u64);
}

pub fn zz_preload() {
    unsafe {
        _zz_preload(NP.as_ptr(), P2.as_ptr());
    }
}
