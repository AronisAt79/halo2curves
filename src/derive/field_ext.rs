#[macro_export]
macro_rules! field_ext_common {
    (
        $field_ext:ident,
        $field:ident,
        $nonresidue:ident,
        $next_nonresidue_0:ident,
        $next_nonresidue_1:ident,
        $size:expr,
        $base_size:expr,
        $base_bits:expr,
        $zeta:ident
    ) => {
        /// An element of the extension field, represented by c0 + c1 * u; where u^2 = U_SQUARE.
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #[cfg_attr(feature = "derive_serde", derive(Serialize, Deserialize))]
        pub struct $field_ext {
            pub c0: $field,
            pub c1: $field,
        }

        impl Ord for $field_ext {
            #[inline(always)]
            fn cmp(&self, other: &$field_ext) -> Ordering {
                match self.c1.cmp(&other.c1) {
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => self.c0.cmp(&other.c0),
                }
            }
        }

        impl PartialOrd for $field_ext {
            #[inline(always)]
            fn partial_cmp(&self, other: &$field_ext) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl ConditionallySelectable for $field_ext {
            fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                $field_ext {
                    c0: $field::conditional_select(&a.c0, &b.c0, choice),
                    c1: $field::conditional_select(&a.c1, &b.c1, choice),
                }
            }
        }

        impl ConstantTimeEq for $field_ext {
            fn ct_eq(&self, other: &Self) -> Choice {
                self.c0.ct_eq(&other.c0) & self.c1.ct_eq(&other.c1)
            }
        }

        impl Default for $field_ext {
            #[inline]
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl From<$field_ext> for [u8; $size] {
            fn from(value: $field_ext) -> [u8; $size] {
                value.to_bytes()
            }
        }

        impl<'a> From<&'a $field_ext> for [u8; $size] {
            fn from(value: &'a $field_ext) -> [u8; $size] {
                value.to_bytes()
            }
        }

        impl Neg for $field_ext {
            type Output = $field_ext;

            #[inline]
            fn neg(self) -> $field_ext {
                -&self
            }
        }

        impl<'a> Neg for &'a $field_ext {
            type Output = $field_ext;

            #[inline]
            fn neg(self) -> $field_ext {
                self.neg()
            }
        }

        impl<'a, 'b> Sub<&'b $field_ext> for &'a $field_ext {
            type Output = $field_ext;

            #[inline]
            fn sub(self, rhs: &'b $field_ext) -> $field_ext {
                self.sub(rhs)
            }
        }

        impl<'a, 'b> Add<&'b $field_ext> for &'a $field_ext {
            type Output = $field_ext;

            #[inline]
            fn add(self, rhs: &'b $field_ext) -> $field_ext {
                self.add(rhs)
            }
        }

        impl<'a, 'b> Mul<&'b $field_ext> for &'a $field_ext {
            type Output = $field_ext;

            #[inline]
            fn mul(self, rhs: &'b $field_ext) -> $field_ext {
                self.mul(rhs)
            }
        }

        /// Extension field size in bytes.
        const SIZE: usize = $size;
        /// Base field size in bytes.
        const COEFF_SIZE: usize = $base_size;

        // Non-residue for the next extension in the tower.
        pub(crate) const V_CUBE: $field_ext = $field_ext {
            c0: V_CUBE_0,
            c1: V_CUBE_1,
        };

        impl $field_ext {
            /// Returns the zero element.
            #[inline]
            pub const fn zero() -> $field_ext {
                $field_ext {
                    c0: $field::zero(),
                    c1: $field::zero(),
                }
            }

            /// Returns the unit element.
            #[inline]
            pub const fn one() -> $field_ext {
                $field_ext {
                    c0: $field::one(),
                    c1: $field::zero(),
                }
            }

            /// Given its base field coefficients c0, c1. Returns the element: c1 * u + c0.
            pub const fn new(c0: $field, c1: $field) -> Self {
                $field_ext { c0, c1 }
            }

            /// Size in bytes of the extension field element.
            pub const fn size() -> usize {
                SIZE
            }

            /// Constructs an field extension element from a little-endian byte representation
            /// of its base field coefficients, failing if their representation is not canonical.
            pub fn from_bytes(bytes: &[u8; SIZE]) -> CtOption<$field_ext> {
                let c0 = $field::from_bytes(bytes[0..COEFF_SIZE].try_into().unwrap());
                let c1 = $field::from_bytes(bytes[COEFF_SIZE..SIZE].try_into().unwrap());
                CtOption::new(
                    $field_ext {
                        c0: c0.unwrap(),
                        c1: c1.unwrap(),
                    },
                    c0.is_some() & c1.is_some(),
                )
            }

            /// Converts an element of field extension into a byte representation.
            /// This representation consists of the base field coefficients in ascending order,
            /// that is, with the independent coefficient first.
            /// Each coeffcient is encoded in little-endian.
            pub fn to_bytes(self) -> [u8; SIZE] {
                let mut res = [0u8; SIZE];
                let c0_bytes = self.c0.to_bytes();
                let c1_bytes = self.c1.to_bytes();
                res[0..COEFF_SIZE].copy_from_slice(&c0_bytes[..]);
                res[COEFF_SIZE..SIZE].copy_from_slice(&c1_bytes[..]);
                res
            }

            /// Computes a = a * b
            pub fn mul_assign(&mut self, other: &Self) {
                // r0 = s0 * s0 + U_SQUARE * s1 * o1
                // r1 = s0 * o1 - s1 * o0
                let t0 = self.c0 * other.c0;
                let t1 = self.c0 * other.c1;
                let t2 = self.c1 * other.c0;
                let t3 = self.c1 * other.c1;

                self.c0 = t0 + $nonresidue * t3;
                self.c1 = t1 + t2
            }

            /// Computes a = a^2
            pub fn square_assign(&mut self) {
                // r0 = s0^2 + U_SQUARE * s1^2
                // r1 = 2* s0s1

                let ab = self.c0 * self.c1;
                let a2 = self.c0 * self.c0;
                let b2 = self.c1 * self.c1;

                self.c1 = ab.double();
                self.c0 = a2 + $nonresidue * b2;
            }

            /// Returns = 2 * a
            pub fn double(&self) -> Self {
                Self {
                    c0: self.c0.double(),
                    c1: self.c1.double(),
                }
            }

            /// Computes a = 2 * a
            pub fn double_assign(&mut self) {
                self.c0 = self.c0.double();
                self.c1 = self.c1.double();
            }

            /// Retruns a + b
            pub fn add(&self, other: &Self) -> Self {
                Self {
                    c0: self.c0.add(&other.c0),
                    c1: self.c1.add(&other.c1),
                }
            }

            /// Retruns a - b
            pub fn sub(&self, other: &Self) -> Self {
                Self {
                    c0: self.c0.sub(&other.c0),
                    c1: self.c1.sub(&other.c1),
                }
            }

            /// Retruns a * b
            pub fn mul(&self, other: &Self) -> Self {
                let mut t = *other;
                t.mul_assign(self);
                t
            }

            /// Retruns a^2
            pub fn square(&self) -> Self {
                let mut t = *self;
                t.square_assign();
                t
            }

            /// Retruns -a
            pub fn neg(&self) -> Self {
                Self {
                    c0: self.c0.neg(),
                    c1: self.c1.neg(),
                }
            }

            ///  Returns the conjugate element. (-c1 * u + c0)
            pub fn conjugate(&mut self) {
                self.c1 = -self.c1;
            }

            /// Returns the frobenius map:
            /// (c1 * u + c0)^(p^power) =
            /// c1 * u^(p*power) + c0
            /// original element if power is even, cojugate if it is odd.
            pub fn frobenius_map(&mut self, power: usize) {
                //  Note: This is not constant time
                //
                //  We can bring the exponent inside the parentheses because
                //  all the cross-terms are 0 mod p.
                //  We remove ^(p*power) from c0 and c1 because for any element of the base field a: a ^ p = a
                //  Finally, u^2 is quadratic nonresidue so u^p = (u^2)^((p-1)/2) * u = -u
                if power % 2 != 0 {
                    self.conjugate()
                }
            }

            /// Multiply this element by cubic nonresidue: V_CUBE.
            /// This must be the element used to construct the next cubic extension.
            pub fn mul_by_nonresidue(&mut self) {
                // (x + y * u) * V_CUBE
                self.mul_assign(&V_CUBE)
            }

            pub fn invert(&self) -> CtOption<Self> {
                let mut t1 = self.c1;
                t1 = t1.square();
                t1 *= $nonresidue;
                let mut t0 = self.c0;
                t0 = t0.square();
                //t0 = c0^2 - U_SQUARE c1^2
                t0 -= &t1;
                t0.invert().map(|t| {
                    let mut tmp = $field_ext {
                        c0: self.c0,
                        c1: self.c1,
                    };
                    tmp.c0 *= &t;
                    tmp.c1 *= &t;
                    tmp.c1 = -tmp.c1;

                    tmp
                })
            }

            /// Norm of extension field element.
            fn norm(&self) -> $field {
                // norm = self * self.cojungate()
                let t0 = self.c0.square();
                let t1 = self.c1.square() * $nonresidue;
                t0 - t1
            }
        }

        impl Legendre for $field_ext {
            fn legendre(&self) -> i64 {
                self.norm().legendre()
            }
        }

        impl From<bool> for $field_ext {
            fn from(bit: bool) -> $field_ext {
                if bit {
                    $field_ext::ONE
                } else {
                    $field_ext::ZERO
                }
            }
        }

        impl From<u64> for $field_ext {
            fn from(val: u64) -> Self {
                $field_ext {
                    c0: $field::from(val),
                    c1: $field::zero(),
                }
            }
        }

        paste::paste! {
        // This trait is only implemented to satisfy the requirement of CurveExt.
        // This is in fact not a prime field.
        impl PrimeField for $field_ext {
            type Repr = [<$field_ext Bytes>];

            const MODULUS: &'static str = MODULUS_STR;
            const MULTIPLICATIVE_GENERATOR: Self = $field_ext {
                c0: $field::MULTIPLICATIVE_GENERATOR,
                c1: $field::ZERO,
            };
            const NUM_BITS: u32 = $base_bits;
            const CAPACITY: u32 = $base_bits - 1;
            const S: u32 = 0;

            const ROOT_OF_UNITY: Self = $field_ext::zero();
            const ROOT_OF_UNITY_INV: Self = $field_ext {
                c0: $field::zero(),
                c1: $field::zero(),
            };
            const DELTA: Self = $field_ext {
                c0: $field::zero(),
                c1: $field::zero(),
            };
            const TWO_INV: Self = $field_ext {
                c0: $field::TWO_INV,
                c1: $field::zero(),
            };

            fn from_repr(repr: Self::Repr) -> CtOption<Self> {
                let c0 = $field::from_bytes(&repr.0[..COEFF_SIZE].try_into().unwrap());
                let c1 = $field::from_bytes(&repr.0[COEFF_SIZE..].try_into().unwrap());
                // Disallow overflow representation
                CtOption::new($field_ext::new(c0.unwrap(), c1.unwrap()), Choice::from(1))
            }

            fn to_repr(&self) -> Self::Repr {
                [<$field_ext Bytes>](self.to_bytes())
            }

            fn is_odd(&self) -> Choice {
                Choice::from(self.to_repr().as_ref()[0] & 1)
            }
        }
        }

        impl FromUniformBytes<64> for $field_ext {
            fn from_uniform_bytes(bytes: &[u8; 64]) -> Self {
                Self::new($field::from_uniform_bytes(bytes), $field::zero())
            }
        }

        paste::paste! {
            /// Canonical little-endian representation of an quadratic extension field element.
            /// First half of the bytes represent `c0`, the second half represent `c1`.
            #[derive(Clone, Copy, Debug)]
            pub struct [<$field_ext Bytes>]([u8; SIZE]);

            impl Default for [<$field_ext Bytes>] {
                fn default() -> Self {
                    Self([0u8; SIZE])
                }
            }

            impl AsMut<[u8]> for [<$field_ext Bytes>] {
                fn as_mut(&mut self) -> &mut [u8] {
                    &mut self.0
                }
            }

            impl AsRef<[u8]> for [<$field_ext Bytes>] {
                fn as_ref(&self) -> &[u8] {
                    &self.0
                }
            }
        }

        impl crate::serde::SerdeObject for $field_ext {
            fn from_raw_bytes_unchecked(bytes: &[u8]) -> Self {
                debug_assert_eq!(bytes.len(), SIZE);
                let [c0, c1] = [0, COEFF_SIZE]
                    .map(|i| $field::from_raw_bytes_unchecked(&bytes[i..i + COEFF_SIZE]));
                Self { c0, c1 }
            }
            fn from_raw_bytes(bytes: &[u8]) -> Option<Self> {
                if bytes.len() != SIZE {
                    return None;
                }
                let [c0, c1] =
                    [0, COEFF_SIZE].map(|i| $field::from_raw_bytes(&bytes[i..i + COEFF_SIZE]));
                c0.zip(c1).map(|(c0, c1)| Self { c0, c1 })
            }
            fn to_raw_bytes(&self) -> Vec<u8> {
                let mut res = Vec::with_capacity(SIZE);
                for limb in self.c0.0.iter().chain(self.c1.0.iter()) {
                    res.extend_from_slice(&limb.to_le_bytes());
                }
                res
            }
            fn read_raw_unchecked<R: std::io::Read>(reader: &mut R) -> Self {
                let [c0, c1] = [(); 2].map(|_| $field::read_raw_unchecked(reader));
                Self { c0, c1 }
            }
            fn read_raw<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                let c0 = $field::read_raw(reader)?;
                let c1 = $field::read_raw(reader)?;
                Ok(Self { c0, c1 })
            }
            fn write_raw<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                self.c0.write_raw(writer)?;
                self.c1.write_raw(writer)
            }
        }

        impl WithSmallOrderMulGroup<3> for $field_ext {
            // Cubic root of unity. ZETA^3 = 1 and ZETA^2 != 1.
            const ZETA: Self = $field_ext {
                c0: $zeta,
                c1: $field::zero(),
            };
        }
    };
}