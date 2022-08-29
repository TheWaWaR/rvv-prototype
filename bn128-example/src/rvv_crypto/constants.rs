use super::{gfp::Gfp, gfp2::Gfp2};

// Notice the constants here are taken from:
// https://github.com/ethereum/go-ethereum/blob/ae8ce7202244621d6e80eb69fcc31683fa0d4cea/crypto/bn256/cloudflare/constants.go
// In other words, they are settings for the ethereum bn128 curve,
// which is different from constants from cloudflare's official repo here:
// https://github.com/cloudflare/bn256/blob/9bd9f73a0273ed2f42707ed13b3e36d38baa2a49/constants.go
pub const NP: [u64; 4] = [
    0x87d20782e4866389,
    0x9ede7d651eca6ac9,
    0xd8afcbd01833da80,
    0xf57a22b791888c6b,
];
pub const P2: [u64; 4] = [
    0x3c208c16d87cfd47,
    0x97816a916871ca8d,
    0xb85045b68181585d,
    0x30644e72e131a029,
];
pub const RN1: [u64; 4] = [
    0xed84884a014afa37,
    0xeb2022850278edf8,
    0xcf63e9cfb74492d9,
    0x2e67157159e5c639,
];
pub const P_MINUS2: [u64; 4] = [
    0x3c208c16d87cfd45,
    0x97816a916871ca8d,
    0xb85045b68181585d,
    0x30644e72e131a029,
];
pub const R3: [u64; 4] = [
    0xb1cd6dafda1530df,
    0x62f210e6a7283db6,
    0xef7f0b0c0ada0afb,
    0x20fd6e902d592544,
];
pub const R2: [u64; 4] = [
    0xf32cfc5b538afa89,
    0xb5e71911d44501fb,
    0x47ab1eff0a417ff6,
    0x06d89f71cab8351f,
];
pub const XI_TO_2P_MINUS2_OVER3: [[u64; 4]; 2] = [
    [
        0x5dddfd154bd8c949,
        0x62cb29a5a4445b60,
        0x37bc870a0c7dd2b9,
        0x24830a9d3171f0fd,
    ],
    [
        0x7361d77f843abe92,
        0xa5bb2bd3273411fb,
        0x9c941f314b3e2399,
        0x15df9cddbb9fd3ec,
    ],
];
pub const XI_TO_P_MINUS1_OVER2: [[u64; 4]; 2] = [
    [
        0xa1d77ce45ffe77c7,
        0x07affd117826d1db,
        0x6d16bd27bb7edc6b,
        0x2c87200285defecc,
    ],
    [
        0xe4bbdd0c2936b629,
        0xbb30f162e133bacb,
        0x31a9d1b6f9645366,
        0x253570bea500f8dd,
    ],
];
pub const XI_TO_P_MINUS1_OVER3: [[u64; 4]; 2] = [
    [
        0x6e849f1ea0aa4757,
        0xaa1c7b6d89f89141,
        0xb6e713cdfae0ca3a,
        0x26694fbb4e82ebc3,
    ],
    [
        0xb5773b104563ab30,
        0x347f91c8a9aa6454,
        0x7a007127242e0991,
        0x1956bcd8118214ec,
    ],
];
pub const XI_TO_2P_SQUARED_MINUS2_OVER3: [u64; 4] = [
    0x71930c11d782e155,
    0xa6bb947cffbe3323,
    0xaa303344d4741444,
    0x2c3b3f0d26594943,
];
pub const XI_TO_P_SQUARED_MINUS1_OVER3: [u64; 4] = [
    0x3350c88e13e80b9c,
    0x7dce557cdb5e56b9,
    0x6001b4b8b615564a,
    0x2682e617020217e0,
];
pub const XI_TO_P_MINUS1_OVER6: [[u64; 4]; 2] = [
    [
        0xa222ae234c492d72,
        0xd00f02a4565de15b,
        0xdc2ff3a253dfc926,
        0x10a75716b3899551,
    ],
    [
        0xaf9ba69633144907,
        0xca6b1d7387afb78a,
        0x11bded5ef08a2087,
        0x02f34d751a1f3a7c,
    ],
];
pub const XI_TO_P_SQUARED_MINUS1_OVER6: [u64; 4] = [
    0xca8d800500fa1bf2,
    0xf0c5d61468b39769,
    0x0e201271ad0d4418,
    0x04290f65bad856e6,
];
pub const U: [u64; 4] = [4965661367192848881, 0, 0, 0];
pub const P: [u64; 4] = [
    4332616871279656263,
    10917124144477883021,
    13281191951274694749,
    3486998266802970665,
];

pub fn constant_to_gfp2(a: &[[u64; 4]; 2]) -> Gfp2 {
    Gfp2([Gfp(a[0]), Gfp(a[1])])
}
