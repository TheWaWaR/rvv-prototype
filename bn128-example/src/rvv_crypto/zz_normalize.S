.text
.globl _zz_normalize
// a0: x
// a1: n
_zz_normalize:
  vsetvli t0, a1, e256, m2
  vle256.v v2, (a0)
  vmsleu.vv v0, v30, v2
  vsub.vv v2, v2, v30, v0.t
  vse256.v v2, (a0)
  ret
