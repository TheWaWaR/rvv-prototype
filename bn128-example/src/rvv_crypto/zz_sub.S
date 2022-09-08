.text
.globl _zz_sub
// a0: ret
// a1: x
// a2: y
// a3: size
_zz_sub:
  vsetvli t0, a3, e256, m2

  vle256.v v2, (a1)
  vle256.v v4, (a2)

  vsub.vv v6, v2, v4
  vmsleu.vv v0, v30, v6
  vadd.vv v6, v6, v30, v0.t
  vse256.v v6, (a0)
  ret
