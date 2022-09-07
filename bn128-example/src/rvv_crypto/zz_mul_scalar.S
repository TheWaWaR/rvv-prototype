.text
.globl _zz_mul_scalar
// Args:
//   a0: ret
//   a1: x
//   a2: y
//   a3: size
_zz_mul_scalar:
  vsetvli t0, a3, e256, m2

  vle256.v v0, (a1)            // v0 -> x
  vlse256.v v2, (a2), zero     // v2 -> y

  // T = x * y
  vwmulu.vv v4, v0, v2  // v4 -> T

  // m = (T * N⁻¹) % R
  vnsrl.wx v0, v4, zero // v0 -> T % 2^512
  vmul.vv v0, v0, v28   // v0 -> m

  // t = (T + m * N) / R
  vwmaccu.vv v4, v0, v30
  li t1, 256
  vnsrl.wx v4, v4, t1   // v4 -> t

  // if (N <= t):
  //    t -= N
  vmsleu.vv v0, v30, v4
  vsub.vv v4, v4, v30, v0.t

  vse256.v v4, (a0)
  ret
