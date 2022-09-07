.text
.globl _zz_mul_indexed
// Args:
//   a0: ret
//   a1: x
//   a2: y
//   a3: x_index
//   a4: y_index
//   a5: n
_zz_mul_indexed:
  vsetvli t0, a5, e256, m2

  vle16.v v4, (a3)
  vluxei16.v v0, (a1), v4
  vle16.v v4, (a4)
  vluxei16.v v2, (a2), v4

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
