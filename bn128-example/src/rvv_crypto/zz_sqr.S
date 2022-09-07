.text
.globl _zz_sqr
// Args:
//   a0: ret
//   a1: x
//   a2: size
_zz_sqr:
  vsetvli t0, a2, e256, m2

  vle256.v v0, (a1)     // v0 -> x

  // T = x * y
  vwmulu.vv v4, v0, v0  // v4 -> T

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
