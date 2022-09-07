.text
.globl _zz_neg
// a0: ret
// a1: x
// a2: size
_zz_neg:
  vsetvli t0, a2, e256, m2
  vle256.v v2, (a1)
  vsub.vv v4, v30, v2
  vmsleu.vv v0, v30, v4
  vsub.vv v4, v4, v30, v0.t
  vse256.v v4, (a0)
  ret
