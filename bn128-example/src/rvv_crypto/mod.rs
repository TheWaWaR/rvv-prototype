mod casts;
mod constants;
pub mod cross_tests;
pub mod curve;
pub mod ethereum_tests;
pub mod gfp;
pub mod gfp12;
pub mod gfp2;
pub mod gfp6;
mod macros;
pub mod miller;
pub mod twist;

use self::{curve::CurvePoint, gfp12::Gfp12, miller::miller, twist::TwistPoint};
use core::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Str(&'static str),
    StrArg1(&'static str, u64),
    StrArg2(&'static str, u64, u64),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Error {
        Error::Str(s)
    }
}

pub fn alt_bn128_pairing(data: &[u8], output: &mut u8) -> Result<(), Error> {
    if data.len() % 192 != 0 {
        return Err("Invalid input length, must be multiple of 192!".into());
    }
    let mut acc = Gfp12::one();
    for i in 0..(data.len() / 192) {
        let g1: CurvePoint = data[i * 192..i * 192 + 64].try_into()?;
        let g2: TwistPoint = data[i * 192 + 64..i * 192 + 192].try_into()?;

        acc.mul_ref(&miller(g2, g1));
    }
    acc.final_exponentiation();
    if acc.is_one() {
        *output = 1;
    } else {
        *output = 0;
    }
    Ok(())
}
