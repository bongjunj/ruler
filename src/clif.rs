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
                "udiv" = Udiv([Id; 2]),
                "sdiv" = Sdiv([Id; 2]),
                "urem" = Urem([Id; 2]),
                "srem" = Srem([Id; 2]),
                "umin" = Umin([Id; 2]),
                "umax" = Umax([Id; 2]),
                "smin" = Smin([Id; 2]),
                "smax" = Smax([Id; 2]),
                "ineg" = Ineg(Id),
                "iabs" = Iabs(Id),
                "bnot" = Bnot(Id),
                "band" = Band([Id; 2]),
                "bor" = Bor([Id; 2]),
                "bxor" = Bxor([Id; 2]),
                "ishl" = Ishl([Id; 2]),
                "ushr" = Ushr([Id; 2]),
                "sshr" = Sshr([Id; 2]),
                "rotl" = Rotl([Id; 2]),
                "rotr" = Rotr([Id; 2]),
                "eq" = Eq([Id; 2]),
                "ne" = Ne([Id; 2]),
                "ule" = Ule([Id; 2]),
                "ult" = Ult([Id; 2]),
                "uge" = Uge([Id; 2]),
                "ugt" = Ugt([Id; 2]),
                "sle" = Sle([Id; 2]),
                "slt" = Slt([Id; 2]),
                "sge" = Sge([Id; 2]),
                "sgt" = Sgt([Id; 2]),
                "clz" = Clz(Id),
                "ctz" = Ctz(Id),
                "cls" = Cls(Id),
                "popcnt" = PopCnt(Id),
                "select" = Select([Id; 3]),
                "k" = K(Id),
                "cf-iadd" = CfIadd([Id; 2]),
                "cf-isub" = CfIsub([Id; 2]),
                "cf-imul" = CfImul([Id; 2]),
                "cf-udiv" = CfUdiv([Id; 2]),
                "cf-sdiv" = CfSdiv([Id; 2]),
                "cf-urem" = CfUrem([Id; 2]),
                "cf-srem" = CfSrem([Id; 2]),
                "cf-umin" = CfUmin([Id; 2]),
                "cf-umax" = CfUmax([Id; 2]),
                "cf-smin" = CfSmin([Id; 2]),
                "cf-smax" = CfSmax([Id; 2]),
                "cf-ineg" = CfIneg(Id),
                "cf-iabs" = CfIabs(Id),
                "cf-bnot" = CfBnot(Id),
                "cf-band" = CfBand([Id; 2]),
                "cf-bor" = CfBor([Id; 2]),
                "cf-bxor" = CfBxor([Id; 2]),
                "cf-ishl" = CfIshl([Id; 2]),
                "cf-ushr" = CfUshr([Id; 2]),
                "cf-sshr" = CfSshr([Id; 2]),
                "cf-rotl" = CfRotl([Id; 2]),
                "cf-rotr" = CfRotr([Id; 2]),
                "cf-eq" = CfEq([Id; 2]),
                "cf-ne" = CfNe([Id; 2]),
                "cf-ule" = CfUle([Id; 2]),
                "cf-ult" = CfUlt([Id; 2]),
                "cf-uge" = CfUge([Id; 2]),
                "cf-ugt" = CfUgt([Id; 2]),
                "cf-sle" = CfSle([Id; 2]),
                "cf-slt" = CfSlt([Id; 2]),
                "cf-sge" = CfSge([Id; 2]),
                "cf-sgt" = CfSgt([Id; 2]),
                "cf-clz" = CfClz(Id),
                "cf-ctz" = CfCtz(Id),
                "cf-cls" = CfCls(Id),
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
                    Clif::K(a) => get_cvec(a).clone(),

                    Clif::Ineg(a) => map!(get_cvec, a => Some(a.wrapping_neg())),
                    Clif::CfIneg(a) => map!(get_cvec, a => Some(a.wrapping_neg())),
                    Clif::Iabs(a) => map!(get_cvec, a => Some(a.iabs())),
                    Clif::CfIabs(a) => map!(get_cvec, a => Some(a.iabs())),
                    Clif::Bnot(a) => map!(get_cvec, a => Some(a.not())),
                    Clif::CfBnot(a) => map!(get_cvec, a => Some(a.not())),

                    Clif::Iadd([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_add(*b))),
                    Clif::CfIadd([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_add(*b))),
                    Clif::Isub([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_sub(*b))),
                    Clif::CfIsub([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_sub(*b))),
                    Clif::Imul([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_mul(*b))),
                    Clif::CfImul([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_mul(*b))),
                    Clif::Udiv([a, b]) => map!(get_cvec, a, b => a.checked_udiv(*b)),
                    Clif::CfUdiv([a, b]) => map!(get_cvec, a, b => a.checked_udiv(*b)),
                    Clif::Sdiv([a, b]) => map!(get_cvec, a, b => a.checked_sdiv(*b)),
                    Clif::CfSdiv([a, b]) => map!(get_cvec, a, b => a.checked_sdiv(*b)),
                    Clif::Urem([a, b]) => map!(get_cvec, a, b => a.checked_urem(*b)),
                    Clif::CfUrem([a, b]) => map!(get_cvec, a, b => a.checked_urem(*b)),
                    Clif::Srem([a, b]) => map!(get_cvec, a, b => a.checked_srem(*b)),
                    Clif::CfSrem([a, b]) => map!(get_cvec, a, b => a.checked_srem(*b)),
                    Clif::Umin([a, b]) => map!(get_cvec, a, b => Some(a.umin(*b))),
                    Clif::CfUmin([a, b]) => map!(get_cvec, a, b => Some(a.umin(*b))),
                    Clif::Umax([a, b]) => map!(get_cvec, a, b => Some(a.umax(*b))),
                    Clif::CfUmax([a, b]) => map!(get_cvec, a, b => Some(a.umax(*b))),
                    Clif::Smin([a, b]) => map!(get_cvec, a, b => Some(a.smin(*b))),
                    Clif::CfSmin([a, b]) => map!(get_cvec, a, b => Some(a.smin(*b))),
                    Clif::Smax([a, b]) => map!(get_cvec, a, b => Some(a.smax(*b))),
                    Clif::CfSmax([a, b]) => map!(get_cvec, a, b => Some(a.smax(*b))),

                    Clif::Band([a, b]) => map!(get_cvec, a, b => Some(*a & *b)),
                    Clif::CfBand([a, b]) => map!(get_cvec, a, b => Some(*a & *b)),
                    Clif::Bor([a, b]) => map!(get_cvec, a, b => Some(*a | *b)),
                    Clif::CfBor([a, b]) => map!(get_cvec, a, b => Some(*a | *b)),
                    Clif::Bxor([a, b]) => map!(get_cvec, a, b => Some(*a ^ *b)),
                    Clif::CfBxor([a, b]) => map!(get_cvec, a, b => Some(*a ^ *b)),

                    Clif::Ishl([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_ishl(*b))),
                    Clif::CfIshl([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_ishl(*b))),
                    Clif::Ushr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_ushr(*b))),
                    Clif::CfUshr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_ushr(*b))),
                    Clif::Sshr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_sshr(*b))),
                    Clif::CfSshr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_sshr(*b))),


                    Clif::Rotl([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_rotl(*b))),
                    Clif::CfRotl([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_rotl(*b))),
                    Clif::Rotr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_rotr(*b))),
                    Clif::CfRotr([a, b]) => map!(get_cvec, a, b => Some(a.wrapping_rotr(*b))),

                    Clif::Eq([a, b]) => map!(get_cvec, a, b => Some(a.bv_eq(*b))),
                    Clif::CfEq([a, b]) => map!(get_cvec, a, b => Some(a.bv_eq(*b))),
                    Clif::Ne([a, b]) => map!(get_cvec, a, b => Some(a.bv_ne(*b))),
                    Clif::CfNe([a, b]) => map!(get_cvec, a, b => Some(a.bv_ne(*b))),
                    Clif::Ule([a, b]) => map!(get_cvec, a, b => Some(a.ule(*b))),
                    Clif::CfUle([a, b]) => map!(get_cvec, a, b => Some(a.ule(*b))),
                    Clif::Ult([a, b]) => map!(get_cvec, a, b => Some(a.ult(*b))),
                    Clif::CfUlt([a, b]) => map!(get_cvec, a, b => Some(a.ult(*b))),
                    Clif::Uge([a, b]) => map!(get_cvec, a, b => Some(a.uge(*b))),
                    Clif::CfUge([a, b]) => map!(get_cvec, a, b => Some(a.uge(*b))),
                    Clif::Ugt([a, b]) => map!(get_cvec, a, b => Some(a.ugt(*b))),
                    Clif::CfUgt([a, b]) => map!(get_cvec, a, b => Some(a.ugt(*b))),
                    Clif::Sle([a, b]) => map!(get_cvec, a, b => Some(a.sle(*b))),
                    Clif::CfSle([a, b]) => map!(get_cvec, a, b => Some(a.sle(*b))),
                    Clif::Slt([a, b]) => map!(get_cvec, a, b => Some(a.slt(*b))),
                    Clif::CfSlt([a, b]) => map!(get_cvec, a, b => Some(a.slt(*b))),
                    Clif::Sge([a, b]) => map!(get_cvec, a, b => Some(a.sge(*b))),
                    Clif::CfSge([a, b]) => map!(get_cvec, a, b => Some(a.sge(*b))),
                    Clif::Sgt([a, b]) => map!(get_cvec, a, b => Some(a.sgt(*b))),
                    Clif::CfSgt([a, b]) => map!(get_cvec, a, b => Some(a.sgt(*b))),

                    Clif::Select([c, a, b]) => get_cvec(c)
                        .iter()
                        .zip(get_cvec(a).iter())
                        .zip(get_cvec(b).iter())
                        .map(|tup| match tup {
                            ((Some(c), a), b) if c.0 != 0 => *a,
                            ((Some(_), _), b) => *b,
                            _ => None,
                        })
                        .collect(),

                    Clif::Clz(a) => map!(get_cvec, a => Some(a.count_leading_zeros())),
                    Clif::CfClz(a) => map!(get_cvec, a => Some(a.count_leading_zeros())),
                    Clif::Ctz(a) => map!(get_cvec, a => Some(a.count_trailing_zeros())),
                    Clif::CfCtz(a) => map!(get_cvec, a => Some(a.count_trailing_zeros())),
                    Clif::Cls(a) => map!(get_cvec, a => Some(a.count_leading_signbits())),
                    Clif::CfCls(a) => map!(get_cvec, a => Some(a.count_leading_signbits())),
                    Clif::PopCnt(a) => map!(get_cvec, a => Some(a.popcnt())),

                    Clif::Lit(n) => vec![Some(n.clone()); cvec_len],
                    Clif::Var(_) => vec![],
                }
            }

            fn mk_interval<'a, F>(&'a self, mut _get_interval: F) -> Interval<Self::Constant>
            where
                F: FnMut(&'a Id) -> &'a Interval<Self::Constant>,
            {
                match self {
                    Clif::Lit(c) => Interval::new(Some(*c), Some(*c)),
                    Clif::K(a) => _get_interval(a).clone(),
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

                #[derive(Clone)]
                struct Z3Value<'a> {
                    value: z3::ast::BV<'a>,
                    defined: z3::ast::Bool<'a>,
                }

                fn z3_true<'a>(ctx: &'a z3::Context) -> z3::ast::Bool<'a> {
                    z3::ast::Bool::from_bool(ctx, true)
                }

                fn z3_and<'a>(
                    ctx: &'a z3::Context,
                    values: &[&z3::ast::Bool<'a>],
                ) -> z3::ast::Bool<'a> {
                    if values.is_empty() {
                        z3_true(ctx)
                    } else {
                        z3::ast::Bool::and(ctx, values)
                    }
                }

                fn shift_amount<'a>(
                    ctx: &'a z3::Context,
                    shift: &z3::ast::BV<'a>,
                ) -> z3::ast::BV<'a> {
                    let mask = z3::ast::BV::from_u64(ctx, ($n - 1) as u64, $n);
                    shift.bvand(&mask)
                }

                fn bool_to_bv<'a>(
                    ctx: &'a z3::Context,
                    cond: &z3::ast::Bool<'a>,
                ) -> z3::ast::BV<'a> {
                    let one = z3::ast::BV::from_u64(ctx, 1, $n);
                    let zero = z3::ast::BV::from_u64(ctx, 0, $n);
                    cond.ite(&one, &zero)
                }

                fn total_unary<'a>(
                    value: z3::ast::BV<'a>,
                    arg: &Z3Value<'a>,
                ) -> Z3Value<'a> {
                    Z3Value {
                        value,
                        defined: arg.defined.clone(),
                    }
                }

                fn total_binary<'a>(
                    ctx: &'a z3::Context,
                    value: z3::ast::BV<'a>,
                    lhs: &Z3Value<'a>,
                    rhs: &Z3Value<'a>,
                ) -> Z3Value<'a> {
                    Z3Value {
                        value,
                        defined: z3_and(ctx, &[&lhs.defined, &rhs.defined]),
                    }
                }

                fn egg_to_z3<'a>(ctx: &'a z3::Context, expr: &[Clif]) -> Z3Value<'a> {
                    let mut buf: Vec<Z3Value> = vec![];
                    for node in expr.as_ref().iter() {
                        match node {
                            Clif::Var(v) => buf.push(Z3Value {
                                value: z3::ast::BV::new_const(ctx, v.to_string(), $n),
                                defined: z3_true(ctx),
                            }),
                            Clif::Lit(c) => buf.push(Z3Value {
                                value: z3::ast::BV::from_u64(ctx, c.0 as u64, $n),
                                defined: z3_true(ctx),
                            }),
                            Clif::Iadd([a, b]) | Clif::CfIadd([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                buf.push(total_binary(ctx, a.value.bvadd(&b.value), a, b))
                            }
                            Clif::Isub([a, b]) | Clif::CfIsub([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                buf.push(total_binary(ctx, a.value.bvsub(&b.value), a, b))
                            }
                            Clif::Imul([a, b]) | Clif::CfImul([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                buf.push(total_binary(ctx, a.value.bvmul(&b.value), a, b))
                            }
                            Clif::Udiv([a, b]) | Clif::CfUdiv([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let zero = z3::ast::BV::from_u64(ctx, 0, $n);
                                let nonzero_divisor = b.value._eq(&zero).not();
                                buf.push(Z3Value {
                                    value: a.value.bvudiv(&b.value),
                                    defined: z3_and(
                                        ctx,
                                        &[&a.defined, &b.defined, &nonzero_divisor],
                                    ),
                                })
                            }
                            Clif::Sdiv([a, b]) | Clif::CfSdiv([a, b])  => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let zero = z3::ast::BV::from_u64(ctx, 0, $n);
                                let min = z3::ast::BV::from_u64(ctx, 1u64 << ($n - 1), $n);
                                let neg_one = z3::ast::BV::from_u64(ctx, u64::MAX, $n);
                                let nonzero_divisor = b.value._eq(&zero).not();
                                let no_overflow =
                                    z3::ast::Bool::and(
                                        ctx,
                                        &[&a.value._eq(&min), &b.value._eq(&neg_one)],
                                    )
                                    .not();
                                buf.push(Z3Value {
                                    value: a.value.bvsdiv(&b.value),
                                    defined: z3_and(
                                        ctx,
                                        &[&a.defined, &b.defined, &nonzero_divisor, &no_overflow],
                                    ),
                                })
                            }

                            Clif::Urem([a, b]) | Clif::CfUrem([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let zero = z3::ast::BV::from_u64(ctx, 0, $n);
                                let nonzero_divisor = b.value._eq(&zero).not();
                                buf.push(Z3Value {
                                    value: a.value.bvurem(&b.value),
                                    defined: z3_and(
                                        ctx,
                                        &[&a.defined, &b.defined, &nonzero_divisor],
                                    ),
                                })
                            }
                            Clif::Srem([a, b]) | Clif::CfSrem([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let zero = z3::ast::BV::from_u64(ctx, 0, $n);
                                let nonzero_divisor = b.value._eq(&zero).not();
                                buf.push(Z3Value {
                                    value: a.value.bvsrem(&b.value),
                                    defined: z3_and(
                                        ctx,
                                        &[&a.defined, &b.defined, &nonzero_divisor],
                                    ),
                                })
                            }
                            Clif::Umin([a, b]) | Clif::CfUmin([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvule(&b.value);
                                buf.push(total_binary(ctx, cond.ite(&a.value, &b.value), a, b))
                            }

                            Clif::Umax([a, b]) | Clif::CfUmax([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvuge(&b.value);
                                buf.push(total_binary(ctx, cond.ite(&a.value, &b.value), a, b))
                            }
                            Clif::Smin([a, b]) | Clif::CfSmin([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvsle(&b.value);
                                buf.push(total_binary(ctx, cond.ite(&a.value, &b.value), a, b))
                            }
                            Clif::Smax([a, b]) | Clif::CfSmax([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvsge(&b.value);
                                buf.push(total_binary(ctx, cond.ite(&a.value, &b.value), a, b))
                            }
                            Clif::Band([a, b]) | Clif::CfBand([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                buf.push(total_binary(ctx, a.value.bvand(&b.value), a, b))
                            }
                            Clif::Bor([a, b]) | Clif::CfBor([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                buf.push(total_binary(ctx, a.value.bvor(&b.value), a, b))
                            }
                            Clif::Bxor([a, b]) | Clif::CfBxor([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                buf.push(total_binary(ctx, a.value.bvxor(&b.value), a, b))
                            }
                            Clif::Ishl([a, b]) | Clif::CfIshl([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let shift = shift_amount(ctx, &b.value);
                                buf.push(total_binary(ctx, a.value.bvshl(&shift), a, b))
                            }
                            Clif::Ushr([a, b]) | Clif::CfUshr([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let shift = shift_amount(ctx, &b.value);
                                buf.push(total_binary(ctx, a.value.bvlshr(&shift), a, b))
                            }
                            Clif::Sshr([a, b]) | Clif::CfSshr([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let shift = shift_amount(ctx, &b.value);
                                buf.push(total_binary(ctx, a.value.bvashr(&shift), a, b))
                            }
                            Clif::Bnot(a) | Clif::CfBnot(a) => {
                                let a = &buf[usize::from(*a)];
                                buf.push(total_unary(a.value.bvnot(), a))
                            }
                            Clif::K(a) => {
                                let a = &buf[usize::from(*a)];
                                buf.push(a.clone())
                            }
                            Clif::Ineg(a) | Clif::CfIneg(a) => {
                                let a = &buf[usize::from(*a)];
                                buf.push(total_unary(a.value.bvneg(), a))
                            }
                            Clif::Iabs(a) | Clif::CfIabs(a) => {
                                let a = &buf[usize::from(*a)];
                                let zero = z3::ast::BV::from_u64(ctx, 0, $n);
                                let nonnegative = a.value.bvsge(&zero);
                                buf.push(total_unary(nonnegative.ite(&a.value, &a.value.bvneg()), a))
                            }

                            // (a << b) | (a >>a (N - b))
                            Clif::Rotl([a, b]) | Clif::CfRotl([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let amount = shift_amount(ctx, &b.value);
                                buf.push(total_binary(ctx, a.value.bvrotl(&amount), a, b));
                            },
                            // (a >> b) | (a <<a (N - b))
                            Clif::Rotr([a, b]) | Clif::CfRotr([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let amount = shift_amount(ctx, &b.value);
                                buf.push(total_binary(ctx, a.value.bvrotr(&amount), a, b));
                            },

                            Clif::Eq([a, b]) | Clif::CfEq([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value._eq(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Ne([a, b]) | Clif::CfNe([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value._eq(&b.value).not();
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Ule([a, b]) | Clif::CfUle([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvule(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Ult([a, b]) | Clif::CfUlt([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvult(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Uge([a, b]) | Clif::CfUge([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvuge(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Ugt([a, b]) | Clif::CfUgt([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvugt(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Sle([a, b]) | Clif::CfSle([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvsle(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Slt([a, b]) | Clif::CfSlt([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvslt(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Sge([a, b]) | Clif::CfSge([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvsge(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Sgt([a, b]) | Clif::CfSgt([a, b]) => {
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = a.value.bvsgt(&b.value);
                                buf.push(total_binary(ctx, bool_to_bv(ctx, &cond), a, b));
                            }
                            Clif::Select([c, a, b]) => {
                                let zero = z3::ast::BV::from_u64(ctx, 0, $n);
                                let c = &buf[usize::from(*c)];
                                let a = &buf[usize::from(*a)];
                                let b = &buf[usize::from(*b)];
                                let cond = c.value._eq(&zero).not();
                                let branch_defined = cond.ite(&a.defined, &b.defined);
                                buf.push(Z3Value {
                                    value: cond.ite(&a.value, &b.value),
                                    defined: z3_and(ctx, &[&c.defined, &branch_defined]),
                                });
                            }

                            // For bit-counting operations, we build structural formulas or conditional sequences:
                            Clif::Clz(a) | Clif::CfClz(a) => {
                                let val = &buf[usize::from(*a)];
                                let mut acc = z3::ast::BV::from_u64(ctx, $n as u64, $n);

                                // Check bit by bit from MSB down to LSB
                                for i in 0..$n {
                                    let bit = val.value.extract(i, i);
                                    let is_one = bit._eq(&z3::ast::BV::from_u64(ctx, 1, 1));
                                    let count_at_i = z3::ast::BV::from_u64(ctx, (($n - 1) - i) as u64, $n);

                                    // If this bit is 1, it overrides previous values because it's the highest '1'
                                    acc = is_one.ite(&count_at_i, &acc);
                                }
                                buf.push(total_unary(acc, val));
                            }

                            Clif::Ctz(a) | Clif::CfCtz(a) => {
                                let val = &buf[usize::from(*a)];
                                let mut acc = z3::ast::BV::from_u64(ctx, $n as u64, $n);

                                // Check bit by bit from LSB up to MSB
                                for i in (0..$n).rev() {
                                    let bit = val.value.extract(i, i);
                                    let is_one = bit._eq(&z3::ast::BV::from_u64(ctx, 1, 1));
                                    let count_at_i = z3::ast::BV::from_u64(ctx, i as u64, $n);

                                    // If this bit is 1, it overrides because it's the lowest '1'
                                    acc = is_one.ite(&count_at_i, &acc);
                                }
                                buf.push(total_unary(acc, val));
                            }

                            Clif::Cls(a) | Clif::CfCls(a) => {
                                let val = &buf[usize::from(*a)];
                                let sign_bit = val.value.extract($n - 1, $n - 1);
                                let mut acc = z3::ast::BV::from_u64(ctx, ($n - 1) as u64, $n);

                                // Check consecutive matching sign bits from MSB-1 down to LSB
                                for i in 0..($n - 1) {
                                    let bit = val.value.extract(i, i);
                                    let matches_sign = bit._eq(&sign_bit);
                                    let count_at_i = z3::ast::BV::from_u64(ctx, (($n - 1) - 1 - i) as u64, $n);

                                    // If the bit differs from the sign bit, we update our fallback accumulator
                                    acc = matches_sign.ite(&acc, &count_at_i);
                                }
                                buf.push(total_unary(acc, val));
                            }

                            Clif::PopCnt(a) => {
                                let val = &buf[usize::from(*a)];

                                // Zero-extend each single bit to size $n, then sum them up
                                let mut sum = z3::ast::BV::from_u64(ctx, 0, $n);
                                for i in 0..$n {
                                    let bit = val.value.extract(i, i).zero_ext($n - 1);
                                    sum = sum.bvadd(&bit);
                                }
                                buf.push(total_unary(sum, val));
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
                let defined_diff = lexpr.defined._eq(&rexpr.defined).not();
                let value_diff_when_defined = z3::ast::Bool::and(
                    &ctx,
                    &[
                        &lexpr.defined,
                        &rexpr.defined,
                        &lexpr.value._eq(&rexpr.value).not(),
                    ],
                );
                solver.assert(&z3::ast::Bool::or(
                    &ctx,
                    &[&defined_diff, &value_diff_when_defined],
                ));
                match solver.check() {
                    SatResult::Sat => ValidationResult::Invalid,
                    SatResult::Unsat => ValidationResult::Valid,
                    SatResult::Unknown => ValidationResult::Unknown,
                }
            }
        }
    };
}
