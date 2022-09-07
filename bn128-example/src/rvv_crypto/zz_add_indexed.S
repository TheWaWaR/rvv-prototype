.text
.globl _zz_add_indexed
// a0: ret
// a1: x
// a2: y
// a3: x_index
// a4: y_index
// a5: n
_zz_add_indexed:
  vsetvli t0, a5, e256, m2

  vle16.v v0, (a3)
  vluxei16.v v2, (a1), v0
  vle16.v v0, (a4)
  vluxei16.v v4, (a2), v0

  vadd.vv v6, v2, v4
  vmsleu.vv v0, v30, v6
  vsub.vv v6, v6, v30, v0.t
  vse256.v v6, (a0)
  ret
