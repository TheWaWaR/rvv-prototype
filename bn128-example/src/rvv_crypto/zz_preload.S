.text
.globl _zz_preload
// a0: NP
// a1: P2
_zz_preload:
  vsetivli t0, 16, e256, m2
  vlse256.v v28, (a0), zero
  vlse256.v v30, (a1), zero
  ret
