macro_rules! gfp_ops_impl {
    ($name:ident) => {
        impl Add for $name {
            type Output = $name;

            fn add(mut self, a: $name) -> $name {
                self.add_ref(&a);
                self
            }
        }

        impl Add<&$name> for $name {
            type Output = $name;

            fn add(mut self, a: &$name) -> $name {
                self.add_ref(a);
                self
            }
        }

        impl Add for &$name {
            type Output = $name;

            fn add(self, a: &$name) -> $name {
                $name::add_to(self, a)
            }
        }

        impl Mul for $name {
            type Output = $name;

            fn mul(mut self, a: $name) -> $name {
                self.mul_ref(&a);
                self
            }
        }

        impl Mul<&$name> for $name {
            type Output = $name;

            fn mul(mut self, a: &$name) -> $name {
                self.mul_ref(a);
                self
            }
        }

        impl Mul for &$name {
            type Output = $name;

            fn mul(self, a: &$name) -> $name {
                $name::mul_to(self, a)
            }
        }

        impl Neg for $name {
            type Output = $name;

            fn neg(mut self) -> $name {
                self.neg_ref();
                self
            }
        }

        impl Neg for &$name {
            type Output = $name;

            fn neg(self) -> $name {
                self.neg_to()
            }
        }

        impl Sub for $name {
            type Output = $name;

            fn sub(mut self, a: $name) -> $name {
                self.sub_ref(&a);
                self
            }
        }

        impl Sub<&$name> for $name {
            type Output = $name;

            fn sub(mut self, a: &$name) -> $name {
                self.sub_ref(a);
                self
            }
        }

        impl Sub for &$name {
            type Output = $name;

            fn sub(self, a: &$name) -> $name {
                $name::sub_to(self, a)
            }
        }

        impl SubAssign for $name {
            fn sub_assign(&mut self, other: $name) {
                self.sub_ref(&other);
            }
        }

        impl SubAssign<&$name> for $name {
            fn sub_assign(&mut self, other: &$name) {
                self.sub_ref(&other);
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, other: $name) {
                self.add_ref(&other);
            }
        }

        impl AddAssign<&$name> for $name {
            fn add_assign(&mut self, other: &$name) {
                self.add_ref(&other);
            }
        }

        impl MulAssign for $name {
            fn mul_assign(&mut self, other: $name) {
                self.mul_ref(&other);
            }
        }

        impl MulAssign<&$name> for $name {
            fn mul_assign(&mut self, other: &$name) {
                self.mul_ref(&other);
            }
        }
    };
}

pub(crate) use gfp_ops_impl;
