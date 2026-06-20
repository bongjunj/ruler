use std::fmt;
use std::ops::*;

use rand::prelude::Distribution;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

// General bitvector implementation
#[derive(Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BV<const N: Inner>(pub Inner);

type Inner = u128;
const INNER_N: Inner = (core::mem::size_of::<Inner>() * 8) as Inner;

impl<const N: Inner> BV<N> {
    pub const ZERO: Self = Self(0);
    pub const ALL_ONES: Self = Self((!(0)) >> (INNER_N - N));
    pub const NEG_ONE: Self = Self::ALL_ONES;
    pub const MIN: Self = Self(1 << (N - 1));
    pub const MAX: Self = Self(Self::ALL_ONES.0 >> 1);

    pub fn new(n: impl Into<Inner>) -> Self {
        Self(n.into() & Self::ALL_ONES.0)
    }

    pub fn wrapping_add(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_add(rhs.0))
    }

    pub fn wrapping_sub(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_sub(rhs.0))
    }

    pub fn wrapping_mul(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_mul(rhs.0))
    }

    pub fn wrapping_neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }

    pub fn iabs(self) -> Self {
        if self.signed() >= 0 {
            self
        } else {
            self.wrapping_neg()
        }
    }

    pub fn my_shl(self, rhs: Self) -> Self {
        if rhs.0 >= N {
            Self::ZERO
        } else {
            Self::new(self.0 << rhs.0)
        }
    }

    pub fn my_shr(self, rhs: Self) -> Self {
        if rhs.0 >= N {
            Self::ZERO
        } else {
            Self::new(self.0 >> rhs.0)
        }
    }

    pub fn wrapping_ishl(self, rhs: Self) -> Self {
        let shift = rhs.0 & (N - 1);
        Self::new(self.0 << shift)
    }

    pub fn wrapping_ushr(self, rhs: Self) -> Self {
        let shift = rhs.0 & (N - 1);
        Self::new(self.0 >> shift)
    }

    pub fn wrapping_sshr(self, rhs: Self) -> Self {
        let shift = (rhs.0 & (N - 1)) as u32;
        let extend = (INNER_N - N) as u32;
        let signed = ((self.0 << extend) as i128) >> extend;
        Self::new((signed >> shift) as u128)
    }

    pub fn wrapping_rotl(self, rhs: Self) -> Self {
        let shift = rhs.0 & (N - 1);
        if shift == 0 {
            self
        } else {
            Self::new((self.0 << shift) | (self.0 >> (N - shift)))
        }
    }

    pub fn wrapping_rotr(self, rhs: Self) -> Self {
        let shift = rhs.0 & (N - 1);
        if shift == 0 {
            self
        } else {
            Self::new((self.0 >> shift) | (self.0 << (N - shift)))
        }
    }

    pub fn count_leading_zeros(self) -> Self {
        let total_bits = 128;
        let padding_bits = total_bits - N;
        // CAUTION: this counts zeros from u128 container
        let zeros = self.0.leading_zeros() as u128;
        let actual_zeros = zeros.saturating_sub(padding_bits);
        Self::new(actual_zeros)
    }

    pub fn count_trailing_zeros(self) -> Self {
        if self.0 == 0 {
            // capped to N
            return Self::new(N);
        }
        let trailing_zeros = self.0.trailing_zeros() as u128;
        Self::new(trailing_zeros)
    }

    pub fn count_leading_signbits(self) -> Self {
        // 1. Get the sign bit of your N-bit integer.
        // Shift right by (N - 1) to see if the MSB is 1 or 0.
        let is_negative = ((self.0 >> (N - 1)) & 1) == 1;

        let zeros = if is_negative {
            // If negative, we care about consecutive 1s.
            // We flip the bits, but we must only look at the valid N bits.
            let inverted = !self.0;

            // Clear out the padding bits at the top of the u128 container
            // so they don't look like leading zeros.
            let mask = (1u128 << N) - 1;
            let valid_inverted = inverted & mask;

            // Count leading zeros of the u128, then subtract the container padding
            valid_inverted.leading_zeros() as u128 - (128 - N)
        } else {
            // If positive, we care about consecutive 0s.
            // Just count leading zeros and subtract the container padding.
            self.0.leading_zeros() as u128 - (128 - N)
        };

        // The definition asks for consecutive bits *after* the sign bit.
        // Since zeros counted the sign bit itself, we subtract 1.
        let result = zeros - 1;

        Self::new(result)
    }

    pub fn popcnt(self) -> Self {
        let bits = self.0 & Self::ALL_ONES.0;
        Self::new(bits.count_ones() as u128)
    }

    pub fn bool(b: bool) -> Self {
        if b {
            Self::new(1u128)
        } else {
            Self::ZERO
        }
    }

    fn signed(self) -> i128 {
        let extend = (INNER_N - N) as u32;
        ((self.0 << extend) as i128) >> extend
    }

    pub fn bv_eq(self, rhs: Self) -> Self {
        Self::bool(self.0 == rhs.0)
    }

    pub fn bv_ne(self, rhs: Self) -> Self {
        Self::bool(self.0 != rhs.0)
    }

    pub fn ule(self, rhs: Self) -> Self {
        Self::bool(self.0 <= rhs.0)
    }

    pub fn ult(self, rhs: Self) -> Self {
        Self::bool(self.0 < rhs.0)
    }

    pub fn uge(self, rhs: Self) -> Self {
        Self::bool(self.0 >= rhs.0)
    }

    pub fn ugt(self, rhs: Self) -> Self {
        Self::bool(self.0 > rhs.0)
    }

    pub fn sle(self, rhs: Self) -> Self {
        Self::bool(self.signed() <= rhs.signed())
    }

    pub fn slt(self, rhs: Self) -> Self {
        Self::bool(self.signed() < rhs.signed())
    }

    pub fn sge(self, rhs: Self) -> Self {
        Self::bool(self.signed() >= rhs.signed())
    }

    pub fn sgt(self, rhs: Self) -> Self {
        Self::bool(self.signed() > rhs.signed())
    }

    pub fn umin(self, rhs: Self) -> Self {
        if self.0 <= rhs.0 {
            self
        } else {
            rhs
        }
    }

    pub fn umax(self, rhs: Self) -> Self {
        if self.0 >= rhs.0 {
            self
        } else {
            rhs
        }
    }

    pub fn smin(self, rhs: Self) -> Self {
        if self.signed() <= rhs.signed() {
            self
        } else {
            rhs
        }
    }

    pub fn smax(self, rhs: Self) -> Self {
        if self.signed() >= rhs.signed() {
            self
        } else {
            rhs
        }
    }

    pub fn select(self, then_val: Self, else_val: Self) -> Self {
        if self.0 != 0 {
            then_val
        } else {
            else_val
        }
    }

    pub fn checked_udiv(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            None
        } else {
            Some(Self::new(self.0 / rhs.0))
        }
    }

    pub fn checked_urem(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            None
        } else {
            Some(Self::new(self.0 % rhs.0))
        }
    }

    pub fn checked_sdiv(self, rhs: Self) -> Option<Self> {
        let lhs = self.signed();
        let rhs = rhs.signed();
        if rhs == 0 || (self == Self::MIN && rhs == -1) {
            None
        } else {
            Some(Self::new((lhs / rhs) as u128))
        }
    }

    pub fn checked_srem(self, rhs: Self) -> Option<Self> {
        let lhs = self.signed();
        let rhs = rhs.signed();
        if rhs == 0 {
            None
        } else if self == Self::MIN && rhs == -1 {
            Some(Self::ZERO)
        } else {
            Some(Self::new((lhs % rhs) as u128))
        }
    }
}

impl<const N: Inner> Not for BV<N> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(self.0.not())
    }
}

impl<const N: Inner> BitAnd for BV<N> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::new(self.0.bitand(rhs.0))
    }
}

impl<const N: Inner> BitOr for BV<N> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::new(self.0.bitor(rhs.0))
    }
}

impl<const N: Inner> BitXor for BV<N> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::new(self.0.bitxor(rhs.0))
    }
}

impl<const N: Inner> fmt::Debug for BV<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<const N: Inner> fmt::Display for BV<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<const N: Inner> Distribution<BV<N>> for rand::distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BV<N> {
        let inner: Inner = rng.gen();
        inner.into()
    }
}

impl<const N: Inner> std::str::FromStr for BV<N> {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("#b") {
            let i = Inner::from_str_radix(stripped, 2).unwrap();
            return Ok(Self::new(i));
        }
        if let Some(stripped) = s.strip_prefix('-') {
            let i = stripped.parse::<Inner>()?;
            return Ok(Self::new(0u128.wrapping_sub(i)));
        }
        s.parse::<Inner>().map(Self::new)
    }
}

impl<const N: Inner> From<Inner> for BV<N> {
    fn from(v: Inner) -> Self {
        Self::new(v)
    }
}

// Macro for specializing BV to different sized bitvectors
#[macro_export]
macro_rules! impl_bv {
    ($n:literal) => {
        use $crate::*;

        use rand::prelude::*;
        use rand_pcg::Pcg64;
        use serde::{Deserialize, Serialize};
        use std::fmt;
        use std::ops::*;

        pub type BV = $crate::BV::<$n>;

        egg::define_language! {
          pub enum Bv {
                  "+" = Add([Id; 2]),
                  "--" = Sub([Id; 2]),
                  "*" = Mul([Id; 2]),
                  "-" = Neg(Id),
                  "~" = Not(Id),
                  "<<" = Shl([Id; 2]),
                  ">>" = Shr([Id; 2]),
                  "&" = And([Id; 2]),
                  "|" = Or([Id; 2]),
                  "^" = Xor([Id; 2]),
                  Lit(BV),
                  Var(egg::Symbol),
              }
        }

        impl SynthLanguage for Bv {
            type Constant = BV;

            fn eval<'a, F>(&'a self, cvec_len: usize, mut get_cvec: F) -> CVec<Self>
            where
                F: FnMut(&'a Id) -> &'a CVec<Self>,
            {
                match self {
                    Bv::Neg(a) => map!(get_cvec, a => Some(a.wrapping_neg())),
                    Bv::Not(a) => map!(get_cvec, a => Some(a.not())),

                    Bv::Add([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_add(*b))),
                    Bv::Sub([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_sub(*b))),
                    Bv::Mul([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_mul(*b))),

                    Bv::Shl([a, b]) => map!(get_cvec, a, b => Some(a.my_shl(*b))),
                    Bv::Shr([a, b]) => map!(get_cvec, a, b => Some(a.my_shr(*b))),

                    Bv::And([a, b]) => map!(get_cvec, a, b => Some(*a & *b)),
                    Bv::Or([a, b]) => map!(get_cvec, a, b => Some(*a | *b)),
                    Bv::Xor([a, b]) => map!(get_cvec, a, b => Some(*a ^ *b)),

                    Bv::Lit(n) => vec![Some(n.clone()); cvec_len],
                    Bv::Var(_) => vec![],
                }
            }

            fn mk_interval<'a, F>(&'a self, mut get_interval: F) -> Interval<Self::Constant>
            where
                F: FnMut(&'a Id) -> &'a Interval<Self::Constant>,
            {
                match self {
                    Bv::Lit(c) => Interval::new(Some(*c), Some(*c)),
                    // Todo- proper interval analysis. For now it's just constant folding
                    _ => Interval::default()
                }
            }

            fn to_var(&self) -> Option<Symbol> {
                if let Bv::Var(sym) = self {
                    Some(*sym)
                } else {
                    None
                }
            }

            fn mk_var(sym: Symbol) -> Self {
                Bv::Var(sym)
            }

            fn is_constant(&self) -> bool {
                matches!(self, Bv::Lit(_))
            }

            fn mk_constant(c: Self::Constant, _egraph: &mut EGraph<Self, SynthAnalysis>) -> Self {
                Bv::Lit(c)
            }

            fn initialize_vars(egraph: &mut EGraph<Self, SynthAnalysis>, vars: &[String]) {
                //   let mut consts: Vec<Option<BV>> = (0..1u64 << $n).map(|i| Some((i as u32).into())).collect();
                let mut consts = vec![];

                for i in 0..2 {
                    let i = BV::from(i);
                    consts.push(Some(BV::MIN.wrapping_add(i)));
                    consts.push(Some(BV::MAX.wrapping_sub(i)));
                    consts.push(Some(i));
                    consts.push(Some(i.wrapping_neg()));
                }
                consts.sort();
                consts.dedup();

                let mut cvecs = self_product(&consts, vars.len());

                egraph.analysis.cvec_len = cvecs[0].len();

                for (i, v) in vars.iter().enumerate() {
                    let id = egraph.add(Bv::Var(Symbol::from(v.clone())));
                    egraph[id].data.cvec = cvecs[i].clone()
                }
            }

            fn validate(
                lhs: &Pattern<Self>,
                rhs: &Pattern<Self>,
            ) -> ValidationResult {
                use z3::{*, ast::Ast};

                fn egg_to_z3<'a>(ctx: &'a z3::Context, expr: &[Bv]) -> z3::ast::BV<'a> {
                    let mut buf: Vec<z3::ast::BV> = vec![];
                    for node in expr.as_ref().iter() {
                        match node {
                            Bv::Var(v) => buf.push(z3::ast::BV::new_const(&ctx, v.to_string(), $n)),
                            Bv::Lit(c) => buf.push(z3::ast::BV::from_u64(&ctx, c.0 as u64, $n)),
                            Bv::Add([a, b]) => buf.push(buf[usize::from(*a)].bvadd(&buf[usize::from(*b)])),
                            Bv::Sub([a, b]) => buf.push(buf[usize::from(*a)].bvsub(&buf[usize::from(*b)])),
                            Bv::Mul([a, b]) => buf.push(buf[usize::from(*a)].bvmul(&buf[usize::from(*b)])),
                            Bv::Shl([a, b]) => buf.push(buf[usize::from(*a)].bvshl(&buf[usize::from(*b)])),
                            Bv::Shr([a, b]) => buf.push(buf[usize::from(*a)].bvlshr(&buf[usize::from(*b)])),
                            Bv::And([a, b]) => buf.push(buf[usize::from(*a)].bvand(&buf[usize::from(*b)])),
                            Bv::Or([a, b]) => buf.push(buf[usize::from(*a)].bvor(&buf[usize::from(*b)])),
                            Bv::Xor([a, b]) => buf.push(buf[usize::from(*a)].bvxor(&buf[usize::from(*b)])),
                            Bv::Not(a) => buf.push(buf[usize::from(*a)].bvnot()),
                            Bv::Neg(a) => buf.push(buf[usize::from(*a)].bvneg()),
                        }
                    }
                    buf.pop().unwrap()
                }

                let mut cfg = z3::Config::new();
                cfg.set_timeout_msec(1000);
                let ctx = z3::Context::new(&cfg);
                let solver = z3::Solver::new(&ctx);
                let lexpr = egg_to_z3(&ctx, Self::instantiate(lhs).as_ref());
                let rexpr = egg_to_z3(&ctx, Self::instantiate(rhs).as_ref());
                solver.assert(&lexpr._eq(&rexpr).not());
                match solver.check() {
                    SatResult::Sat => ValidationResult::Invalid,
                    SatResult::Unsat => ValidationResult::Valid,
                    SatResult::Unknown => ValidationResult::Unknown
                }
            }
        }
    };
}

#[cfg(test)]
pub mod tests {
    use super::*;
    type BV4 = BV<4>;

    #[test]
    fn test_bv() {
        assert_eq!(BV4::ALL_ONES.0, 0b1111);
        assert_eq!(BV4::MAX.0, 0b0111);
        assert_eq!(BV4::MIN.0, 0b1000);

        let one = BV4::from(1);

        assert_eq!(BV4::MAX.wrapping_add(one), BV::MIN);
        assert_eq!(BV4::NEG_ONE.wrapping_neg(), one);
        assert_eq!(BV4::MIN.wrapping_mul(BV::NEG_ONE), BV::MIN);
        assert_eq!(BV4::MIN.wrapping_neg(), BV::MIN);
    }
}
