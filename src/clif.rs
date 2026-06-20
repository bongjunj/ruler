#[macro_export]
macro_rules! impl_clif_bv {
    ($n:literal) => {
        use $crate::*;

        use std::fmt;
        use std::ops::*;

        pub type BV = $crate::BV::<$n>;

        egg::define_language! {
            pub enum Clif {
                "iadd" = Iadd([Id; 2]),
                "isub" = Isub([Id; 2]),
                "imul" = Imul([Id; 2]),
                "ineg" = Ineg(Id),
                "bnot" = Bnot(Id),
                "band" = Band([Id; 2]),
                "bor" = Bor([Id; 2]),
                "bxor" = Bxor([Id; 2]),
                "ishl" = Ishl([Id; 2]),
                "ushr" = Ushr([Id; 2]),
                "sshr" = Sshr([Id; 2]),
                "rotl" = Rotl([Id; 2]),
                "rotr" = Rotr([Id; 2]),
                "clz" = Clz(Id),
                "ctz" = Ctz(Id),
                "cls" = Cls(Id),
                "popcnt" = PopCnt(Id),
                Lit(BV),
                Var(egg::Symbol),
            }
        }

        impl SynthLanguage for Clif {
            type Constant = BV;

            fn eval<'a, F>(&'a self, cvec_len: usize, mut get_cvec: F) -> CVec<Self>
            where
                F: FnMut(&'a Id) -> &'a CVec<Self>,
            {
                match self {
                    Clif::Ineg(a) => map!(get_cvec, a => Some(a.wrapping_neg())),
                    Clif::Bnot(a) => map!(get_cvec, a => Some(a.not())),

                    Clif::Iadd([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_add(*b))),
                    Clif::Isub([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_sub(*b))),
                    Clif::Imul([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_mul(*b))),

                    Clif::Band([a, b]) => map!(get_cvec, a, b => Some(*a & *b)),
                    Clif::Bor([a, b]) => map!(get_cvec, a, b => Some(*a | *b)),
                    Clif::Bxor([a, b]) => map!(get_cvec, a, b => Some(*a ^ *b)),

                    Clif::Ishl([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_ishl(*b))),
                    Clif::Ushr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_ushr(*b))),
                    Clif::Sshr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_sshr(*b))),


                    Clif::Rotl([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_rotl(*b))),
                    Clif::Rotr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_rotr(*b))),

                    Clif::Clz(a) => map!(get_cvec, a => Some(a.count_leading_zeros())),
                    Clif::Ctz(a) => map!(get_cvec, a => Some(a.count_trailing_zeros())),
                    Clif::Cls(a) => map!(get_cvec, a => Some(a.count_leading_signbits())),
                    Clif::PopCnt(a) => map!(get_cvec, a => Some(a.popcnt())),

                    Clif::Lit(n) => vec![Some(n.clone()); cvec_len],
                    Clif::Var(_) => vec![],
                }
            }

            fn mk_interval<'a, F>(&'a self, _get_interval: F) -> Interval<Self::Constant>
            where
                F: FnMut(&'a Id) -> &'a Interval<Self::Constant>,
            {
                match self {
                    Clif::Lit(c) => Interval::new(Some(*c), Some(*c)),
                    _ => Interval::default(),
                }
            }

            fn to_var(&self) -> Option<Symbol> {
                if let Clif::Var(sym) = self {
                    Some(*sym)
                } else {
                    None
                }
            }

            fn mk_var(sym: Symbol) -> Self {
                Clif::Var(sym)
            }

            fn is_constant(&self) -> bool {
                matches!(self, Clif::Lit(_))
            }

            fn mk_constant(c: Self::Constant, _egraph: &mut EGraph<Self, SynthAnalysis>) -> Self {
                Clif::Lit(c)
            }

            fn initialize_vars(egraph: &mut EGraph<Self, SynthAnalysis>, vars: &[String]) {
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

                let cvecs = self_product(&consts, vars.len());

                egraph.analysis.cvec_len = cvecs[0].len();

                for (i, v) in vars.iter().enumerate() {
                    let id = egraph.add(Clif::Var(Symbol::from(v.clone())));
                    egraph[id].data.cvec = cvecs[i].clone()
                }
            }

            fn validate(lhs: &Pattern<Self>, rhs: &Pattern<Self>) -> ValidationResult {
                use z3::{ast::Ast, *};

                fn shift_amount<'a>(
                    ctx: &'a z3::Context,
                    shift: &z3::ast::BV<'a>,
                ) -> z3::ast::BV<'a> {
                    let mask = z3::ast::BV::from_u64(ctx, ($n - 1) as u64, $n);
                    shift.bvand(&mask)
                }

                fn egg_to_z3<'a>(ctx: &'a z3::Context, expr: &[Clif]) -> z3::ast::BV<'a> {
                    let mut buf: Vec<z3::ast::BV> = vec![];
                    for node in expr.as_ref().iter() {
                        match node {
                            Clif::Var(v) => {
                                buf.push(z3::ast::BV::new_const(ctx, v.to_string(), $n))
                            }
                            Clif::Lit(c) => {
                                buf.push(z3::ast::BV::from_u64(ctx, c.0 as u64, $n))
                            }
                            Clif::Iadd([a, b]) => {
                                buf.push(buf[usize::from(*a)].bvadd(&buf[usize::from(*b)]))
                            }
                            Clif::Isub([a, b]) => {
                                buf.push(buf[usize::from(*a)].bvsub(&buf[usize::from(*b)]))
                            }
                            Clif::Imul([a, b]) => {
                                buf.push(buf[usize::from(*a)].bvmul(&buf[usize::from(*b)]))
                            }
                            Clif::Band([a, b]) => {
                                buf.push(buf[usize::from(*a)].bvand(&buf[usize::from(*b)]))
                            }
                            Clif::Bor([a, b]) => {
                                buf.push(buf[usize::from(*a)].bvor(&buf[usize::from(*b)]))
                            }
                            Clif::Bxor([a, b]) => {
                                buf.push(buf[usize::from(*a)].bvxor(&buf[usize::from(*b)]))
                            }
                            Clif::Ishl([a, b]) => {
                                let shift = shift_amount(ctx, &buf[usize::from(*b)]);
                                buf.push(buf[usize::from(*a)].bvshl(&shift))
                            }
                            Clif::Ushr([a, b]) => {
                                let shift = shift_amount(ctx, &buf[usize::from(*b)]);
                                buf.push(buf[usize::from(*a)].bvlshr(&shift))
                            }
                            Clif::Sshr([a, b]) => {
                                let shift = shift_amount(ctx, &buf[usize::from(*b)]);
                                buf.push(buf[usize::from(*a)].bvashr(&shift))
                            }
                            Clif::Bnot(a) => buf.push(buf[usize::from(*a)].bvnot()),
                            Clif::Ineg(a) => buf.push(buf[usize::from(*a)].bvneg()),

                            // (a << b) | (a >>a (N - b))
                            Clif::Rotl([a, b]) => {
                                let amount = shift_amount(ctx, &buf[usize::from(*b)]);
                                buf.push(buf[usize::from(*a)].bvrotl(&amount));
                            },
                            // (a >> b) | (a <<a (N - b))
                            Clif::Rotr([a, b]) => {
                                let amount = shift_amount(ctx, &buf[usize::from(*b)]);
                                buf.push(buf[usize::from(*a)].bvrotr(&amount));
                            },

                            // For bit-counting operations, we build structural formulas or conditional sequences:
                            Clif::Clz(a) => {
                                let val = &buf[usize::from(*a)];
                                let mut acc = z3::ast::BV::from_u64(ctx, $n as u64, $n);

                                // Check bit by bit from MSB down to LSB
                                for i in 0..$n {
                                    let bit = val.extract(i, i);
                                    let is_one = bit._eq(&z3::ast::BV::from_u64(ctx, 1, 1));
                                    let count_at_i = z3::ast::BV::from_u64(ctx, (($n - 1) - i) as u64, $n);

                                    // If this bit is 1, it overrides previous values because it's the highest '1'
                                    acc = is_one.ite(&count_at_i, &acc);
                                }
                                buf.push(acc);
                            }

                            Clif::Ctz(a) => {
                                let val = &buf[usize::from(*a)];
                                let mut acc = z3::ast::BV::from_u64(ctx, $n as u64, $n);

                                // Check bit by bit from LSB up to MSB
                                for i in (0..$n).rev() {
                                    let bit = val.extract(i, i);
                                    let is_one = bit._eq(&z3::ast::BV::from_u64(ctx, 1, 1));
                                    let count_at_i = z3::ast::BV::from_u64(ctx, i as u64, $n);

                                    // If this bit is 1, it overrides because it's the lowest '1'
                                    acc = is_one.ite(&count_at_i, &acc);
                                }
                                buf.push(acc);
                            }

                            Clif::Cls(a) => {
                                let val = &buf[usize::from(*a)];
                                let sign_bit = val.extract($n - 1, $n - 1);
                                let mut acc = z3::ast::BV::from_u64(ctx, ($n - 1) as u64, $n);

                                // Check consecutive matching sign bits from MSB-1 down to LSB
                                for i in 0..($n - 1) {
                                    let bit = val.extract(i, i);
                                    let matches_sign = bit._eq(&sign_bit);
                                    let count_at_i = z3::ast::BV::from_u64(ctx, (($n - 1) - 1 - i) as u64, $n);

                                    // If the bit differs from the sign bit, we update our fallback accumulator
                                    acc = matches_sign.ite(&acc, &count_at_i);
                                }
                                buf.push(acc);
                            }

                            Clif::PopCnt(a) => {
                                let val = &buf[usize::from(*a)];

                                // Zero-extend each single bit to size $n, then sum them up
                                let mut sum = z3::ast::BV::from_u64(ctx, 0, $n);
                                for i in 0..$n {
                                    let bit = val.extract(i, i).zero_ext($n - 1);
                                    sum = sum.bvadd(&bit);
                                }
                                buf.push(sum);
                            }
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
                    SatResult::Unknown => ValidationResult::Unknown,
                }
            }
        }
    };
}
