use std::fmt;

#[derive(Copy, Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ClifBV(pub i128);

impl ClifBV {
    pub fn to_bv<const N: u128>(self) -> crate::BV<N> {
        if self.0 < 0 {
            crate::BV::new(0u128.wrapping_sub(self.0.wrapping_abs() as u128))
        } else {
            crate::BV::new(self.0 as u128)
        }
    }

    pub fn from_bv<const N: u128>(value: crate::BV<N>) -> Self {
        Self(value.0 as i128)
    }

    pub fn to_z3<'a>(self, ctx: &'a z3::Context, bits: u32) -> z3::ast::BV<'a> {
        if self.0 < 0 {
            z3::ast::BV::from_i64(ctx, self.0 as i64, bits)
        } else {
            z3::ast::BV::from_u64(ctx, self.0 as u64, bits)
        }
    }
}

impl fmt::Display for ClifBV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl std::str::FromStr for ClifBV {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("#b") {
            return i128::from_str_radix(stripped, 2).map(Self);
        }
        s.parse::<i128>().map(Self)
    }
}

#[macro_export]
macro_rules! impl_clif_bv {
    ($n:literal) => {
        use $crate::*;

        use std::fmt;
        use std::ops::*;

        pub type BV = $crate::BV<$n>;

        egg::define_language! {
            pub enum Clif {
                "iadd" = Iadd([Id; 3]),
                "isub" = Isub([Id; 3]),
                "imul" = Imul([Id; 3]),
                "udiv" = Udiv([Id; 3]),
                "sdiv" = Sdiv([Id; 3]),
                "urem" = Urem([Id; 3]),
                "srem" = Srem([Id; 3]),
                "umin" = Umin([Id; 3]),
                "umax" = Umax([Id; 3]),
                "smin" = Smin([Id; 3]),
                "smax" = Smax([Id; 3]),
                "ineg" = Ineg([Id; 2]),
                "iabs" = Iabs([Id; 2]),
                "bnot" = Bnot([Id; 2]),
                "band" = Band([Id; 3]),
                "bor" = Bor([Id; 3]),
                "bxor" = Bxor([Id; 3]),
                "ishl" = Ishl([Id; 3]),
                "ushr" = Ushr([Id; 3]),
                "sshr" = Sshr([Id; 3]),
                "rotl" = Rotl([Id; 3]),
                "rotr" = Rotr([Id; 3]),
                "eq" = Eq([Id; 3]),
                "ne" = Ne([Id; 3]),
                "ule" = Ule([Id; 3]),
                "ult" = Ult([Id; 3]),
                "uge" = Uge([Id; 3]),
                "ugt" = Ugt([Id; 3]),
                "sle" = Sle([Id; 3]),
                "slt" = Slt([Id; 3]),
                "sge" = Sge([Id; 3]),
                "sgt" = Sgt([Id; 3]),
                "select" = Select([Id; 4]),
                "clz" = Clz([Id; 2]),
                "ctz" = Ctz([Id; 2]),
                "cls" = Cls([Id; 2]),
                "popcnt" = PopCnt([Id; 2]),
                "uextend" = Uextend([Id; 2]),
                "sextend" = Sextend([Id; 2]),
                "ireduce" = Ireduce([Id; 2]),
                "ty" = Ty,
                "ty1" = Ty1,
                "ty2" = Ty2,
                Lit($crate::ClifBV),
                Var(egg::Symbol),
            }
        }

        impl SynthLanguage for Clif {
            type Constant = $crate::ClifBV;

            fn eval<'a, F>(&'a self, cvec_len: usize, mut get_cvec: F) -> CVec<Self>
            where
                F: FnMut(&'a Id) -> &'a CVec<Self>,
            {
                fn eval_unary<'a, F>(a: &'a Id, mut get_cvec: F, f: impl Fn(BV) -> BV) -> CVec<Clif>
                where
                    F: FnMut(&'a Id) -> &'a CVec<Clif>,
                {
                    get_cvec(a)
                        .iter()
                        .map(|a| a.map(|a| $crate::ClifBV::from_bv(f(a.to_bv::<$n>()))))
                        .collect()
                }

                fn eval_binary<'a, F>(
                    a: &'a Id,
                    b: &'a Id,
                    mut get_cvec: F,
                    f: impl Fn(BV, BV) -> Option<BV>,
                ) -> CVec<Clif>
                where
                    F: FnMut(&'a Id) -> &'a CVec<Clif>,
                {
                    get_cvec(a)
                        .iter()
                        .zip(get_cvec(b).iter())
                        .map(|(a, b)| match (a, b) {
                            (Some(a), Some(b)) => {
                                f(a.to_bv::<$n>(), b.to_bv::<$n>()).map($crate::ClifBV::from_bv)
                            }
                            _ => None,
                        })
                        .collect()
                }

                match self {
                    Clif::Ineg([_, a]) => eval_unary(a, get_cvec, |a| a.wrapping_neg()),
                    Clif::Iabs([_, a]) => eval_unary(a, get_cvec, |a| a.iabs()),
                    Clif::Bnot([_, a]) => eval_unary(a, get_cvec, |a| a.not()),

                    Clif::Iadd([_, a, b]) => {
                        eval_binary(a, b, get_cvec, |a, b| Some(a.wrapping_add(b)))
                    }
                    Clif::Isub([_, a, b]) => {
                        eval_binary(a, b, get_cvec, |a, b| Some(a.wrapping_sub(b)))
                    }
                    Clif::Imul([_, a, b]) => {
                        eval_binary(a, b, get_cvec, |a, b| Some(a.wrapping_mul(b)))
                    }
                    Clif::Udiv([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| a.checked_udiv(b)),
                    Clif::Sdiv([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| a.checked_sdiv(b)),
                    Clif::Urem([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| a.checked_urem(b)),
                    Clif::Srem([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| a.checked_srem(b)),
                    Clif::Umin([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.umin(b))),
                    Clif::Umax([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.umax(b))),
                    Clif::Smin([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.smin(b))),
                    Clif::Smax([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.smax(b))),

                    Clif::Band([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a & b)),
                    Clif::Bor([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a | b)),
                    Clif::Bxor([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a ^ b)),

                    Clif::Ishl([_, a, b]) => {
                        eval_binary(a, b, get_cvec, |a, b| Some(a.wrapping_ishl(b)))
                    }
                    Clif::Ushr([_, a, b]) => {
                        eval_binary(a, b, get_cvec, |a, b| Some(a.wrapping_ushr(b)))
                    }
                    Clif::Sshr([_, a, b]) => {
                        eval_binary(a, b, get_cvec, |a, b| Some(a.wrapping_sshr(b)))
                    }

                    Clif::Rotl([_, a, b]) => {
                        eval_binary(a, b, get_cvec, |a, b| Some(a.wrapping_rotl(b)))
                    }
                    Clif::Rotr([_, a, b]) => {
                        eval_binary(a, b, get_cvec, |a, b| Some(a.wrapping_rotr(b)))
                    }

                    Clif::Eq([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.bv_eq(b))),
                    Clif::Ne([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.bv_ne(b))),
                    Clif::Ule([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.ule(b))),
                    Clif::Ult([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.ult(b))),
                    Clif::Uge([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.uge(b))),
                    Clif::Ugt([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.ugt(b))),
                    Clif::Sle([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.sle(b))),
                    Clif::Slt([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.slt(b))),
                    Clif::Sge([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.sge(b))),
                    Clif::Sgt([_, a, b]) => eval_binary(a, b, get_cvec, |a, b| Some(a.sgt(b))),
                    Clif::Select([_, c, a, b]) => get_cvec(c)
                        .iter()
                        .zip(get_cvec(a).iter())
                        .zip(get_cvec(b).iter())
                        .map(|tup| match tup {
                            ((Some(c), a), b) if c.0 != 0 => *a,
                            ((Some(_), _), b) => *b,
                            _ => None,
                        })
                        .collect(),

                    Clif::Clz([_, a]) => eval_unary(a, get_cvec, |a| a.count_leading_zeros()),
                    Clif::Ctz([_, a]) => eval_unary(a, get_cvec, |a| a.count_trailing_zeros()),
                    Clif::Cls([_, a]) => eval_unary(a, get_cvec, |a| a.count_leading_signbits()),
                    Clif::PopCnt([_, a]) => eval_unary(a, get_cvec, |a| a.popcnt()),
                    Clif::Uextend([_, a]) | Clif::Sextend([_, a]) | Clif::Ireduce([_, a]) => {
                        eval_unary(a, get_cvec, |a| a)
                    }

                    Clif::Lit(n) => {
                        vec![Some($crate::ClifBV::from_bv(n.to_bv::<$n>())); cvec_len]
                    }
                    Clif::Ty | Clif::Ty1 | Clif::Ty2 => vec![],
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
                    consts.push(Some($crate::ClifBV::from_bv(BV::MIN.wrapping_add(i))));
                    consts.push(Some($crate::ClifBV::from_bv(BV::MAX.wrapping_sub(i))));
                    consts.push(Some($crate::ClifBV::from_bv(i)));
                    consts.push(Some($crate::ClifBV::from_bv(i.wrapping_neg())));
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
                    bits: u32,
                ) -> z3::ast::BV<'a> {
                    let mask = z3::ast::BV::from_u64(ctx, (bits - 1) as u64, bits);
                    shift.bvand(&mask)
                }

                fn bool_to_bv<'a>(
                    ctx: &'a z3::Context,
                    cond: &z3::ast::Bool<'a>,
                    bits: u32,
                ) -> z3::ast::BV<'a> {
                    let one = z3::ast::BV::from_u64(ctx, 1, bits);
                    let zero = z3::ast::BV::from_u64(ctx, 0, bits);
                    cond.ite(&one, &zero)
                }

                fn total_unary<'a>(value: z3::ast::BV<'a>, arg: &Z3Value<'a>) -> Z3Value<'a> {
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

                fn value_width(value: &z3::ast::BV) -> u32 {
                    value.get_size()
                }

                fn check_expected(actual: u32, expected: Option<u32>) -> Result<(), ()> {
                    if let Some(expected) = expected {
                        if actual != expected {
                            return Err(());
                        }
                    }
                    Ok(())
                }

                fn type_bits(node: &Clif, types: &HashMap<&'static str, u32>) -> Result<u32, ()> {
                    match node {
                        Clif::Ty => types.get("ty").copied().ok_or(()),
                        Clif::Ty1 => types.get("ty1").copied().ok_or(()),
                        Clif::Ty2 => types.get("ty2").copied().ok_or(()),
                        _ => Err(()),
                    }
                }

                fn eval_binary<'a>(
                    ctx: &'a z3::Context,
                    expr: &egg::RecExpr<Clif>,
                    a: Id,
                    b: Id,
                    bits: u32,
                    types: &HashMap<&'static str, u32>,
                    var_widths: &HashMap<String, u32>,
                    literal_bits: u32,
                    seen_vars: &mut HashMap<String, u32>,
                    f: impl Fn(&z3::ast::BV<'a>, &z3::ast::BV<'a>) -> z3::ast::BV<'a>,
                ) -> Result<Z3Value<'a>, ()> {
                    let a = eval_node(
                        ctx,
                        expr,
                        a,
                        Some(bits),
                        types,
                        var_widths,
                        literal_bits,
                        seen_vars,
                    )?;
                    let b = eval_node(
                        ctx,
                        expr,
                        b,
                        Some(bits),
                        types,
                        var_widths,
                        literal_bits,
                        seen_vars,
                    )?;
                    if value_width(&a.value) != bits || value_width(&b.value) != bits {
                        return Err(());
                    }
                    Ok(total_binary(ctx, f(&a.value, &b.value), &a, &b))
                }

                fn eval_node<'a>(
                    ctx: &'a z3::Context,
                    expr: &egg::RecExpr<Clif>,
                    id: Id,
                    expected: Option<u32>,
                    types: &HashMap<&'static str, u32>,
                    var_widths: &HashMap<String, u32>,
                    literal_bits: u32,
                    seen_vars: &mut HashMap<String, u32>,
                ) -> Result<Z3Value<'a>, ()> {
                    let node = &expr[id];
                    match node {
                        Clif::Ty | Clif::Ty1 | Clif::Ty2 => Err(()),
                        Clif::Var(v) => {
                            let name = v.to_string();
                            let assigned = var_widths.get(&name).copied();
                            if let (Some(expected), Some(assigned)) = (expected, assigned) {
                                if expected != assigned {
                                    return Err(());
                                }
                            }
                            let bits = expected.or(assigned).unwrap_or(literal_bits);
                            match seen_vars.get(&name).copied() {
                                Some(prev) if prev != bits => return Err(()),
                                Some(_) => {}
                                None => {
                                    seen_vars.insert(name.clone(), bits);
                                }
                            }
                            Ok(Z3Value {
                                value: z3::ast::BV::new_const(ctx, name, bits),
                                defined: z3_true(ctx),
                            })
                        }
                        Clif::Lit(c) => {
                            let bits = expected.unwrap_or(literal_bits);
                            Ok(Z3Value {
                                value: c.to_z3(ctx, bits),
                                defined: z3_true(ctx),
                            })
                        }
                        Clif::Iadd([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvadd(b),
                            )
                        }
                        Clif::Isub([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvsub(b),
                            )
                        }
                        Clif::Imul([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvmul(b),
                            )
                        }
                        Clif::Udiv([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let zero = z3::ast::BV::from_u64(ctx, 0, bits);
                            let nonzero_divisor = b.value._eq(&zero).not();
                            Ok(Z3Value {
                                value: a.value.bvudiv(&b.value),
                                defined: z3_and(ctx, &[&a.defined, &b.defined, &nonzero_divisor]),
                            })
                        }
                        Clif::Sdiv([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let zero = z3::ast::BV::from_u64(ctx, 0, bits);
                            let min = z3::ast::BV::from_u64(ctx, 1u64 << (bits - 1), bits);
                            let neg_one = z3::ast::BV::from_u64(ctx, u64::MAX, bits);
                            let nonzero_divisor = b.value._eq(&zero).not();
                            let no_overflow = z3::ast::Bool::and(
                                ctx,
                                &[&a.value._eq(&min), &b.value._eq(&neg_one)],
                            )
                            .not();
                            Ok(Z3Value {
                                value: a.value.bvsdiv(&b.value),
                                defined: z3_and(
                                    ctx,
                                    &[&a.defined, &b.defined, &nonzero_divisor, &no_overflow],
                                ),
                            })
                        }
                        Clif::Urem([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let zero = z3::ast::BV::from_u64(ctx, 0, bits);
                            let nonzero_divisor = b.value._eq(&zero).not();
                            Ok(Z3Value {
                                value: a.value.bvurem(&b.value),
                                defined: z3_and(ctx, &[&a.defined, &b.defined, &nonzero_divisor]),
                            })
                        }
                        Clif::Srem([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let zero = z3::ast::BV::from_u64(ctx, 0, bits);
                            let nonzero_divisor = b.value._eq(&zero).not();
                            Ok(Z3Value {
                                value: a.value.bvsrem(&b.value),
                                defined: z3_and(ctx, &[&a.defined, &b.defined, &nonzero_divisor]),
                            })
                        }
                        Clif::Umin([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvule(b).ite(a, b),
                            )
                        }
                        Clif::Umax([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvuge(b).ite(a, b),
                            )
                        }
                        Clif::Smin([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvsle(b).ite(a, b),
                            )
                        }
                        Clif::Smax([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvsge(b).ite(a, b),
                            )
                        }
                        Clif::Band([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvand(b),
                            )
                        }
                        Clif::Bor([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvor(b),
                            )
                        }
                        Clif::Bxor([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| a.bvxor(b),
                            )
                        }
                        Clif::Ishl([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let shift = shift_amount(ctx, &b.value, bits);
                            Ok(total_binary(ctx, a.value.bvshl(&shift), &a, &b))
                        }
                        Clif::Ushr([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let shift = shift_amount(ctx, &b.value, bits);
                            Ok(total_binary(ctx, a.value.bvlshr(&shift), &a, &b))
                        }
                        Clif::Sshr([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let shift = shift_amount(ctx, &b.value, bits);
                            Ok(total_binary(ctx, a.value.bvashr(&shift), &a, &b))
                        }
                        Clif::Rotl([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let amount = shift_amount(ctx, &b.value, bits);
                            Ok(total_binary(ctx, a.value.bvrotl(&amount), &a, &b))
                        }
                        Clif::Rotr([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let amount = shift_amount(ctx, &b.value, bits);
                            Ok(total_binary(ctx, a.value.bvrotr(&amount), &a, &b))
                        }
                        Clif::Eq([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a._eq(b), bits),
                            )
                        }
                        Clif::Ne([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a._eq(b).not(), bits),
                            )
                        }
                        Clif::Ule([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a.bvule(b), bits),
                            )
                        }
                        Clif::Ult([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a.bvult(b), bits),
                            )
                        }
                        Clif::Uge([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a.bvuge(b), bits),
                            )
                        }
                        Clif::Ugt([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a.bvugt(b), bits),
                            )
                        }
                        Clif::Sle([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a.bvsle(b), bits),
                            )
                        }
                        Clif::Slt([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a.bvslt(b), bits),
                            )
                        }
                        Clif::Sge([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a.bvsge(b), bits),
                            )
                        }
                        Clif::Sgt([ty, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            eval_binary(
                                ctx,
                                expr,
                                *a,
                                *b,
                                bits,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                                |a, b| bool_to_bv(ctx, &a.bvsgt(b), bits),
                            )
                        }
                        Clif::Bnot([ty, a]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            Ok(total_unary(a.value.bvnot(), &a))
                        }
                        Clif::Ineg([ty, a]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            Ok(total_unary(a.value.bvneg(), &a))
                        }
                        Clif::Iabs([ty, a]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let zero = z3::ast::BV::from_u64(ctx, 0, bits);
                            let nonnegative = a.value.bvsge(&zero);
                            Ok(total_unary(nonnegative.ite(&a.value, &a.value.bvneg()), &a))
                        }
                        Clif::Clz([ty, a]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let val = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let mut acc = z3::ast::BV::from_u64(ctx, bits as u64, bits);
                            for i in 0..bits {
                                let bit = val.value.extract(i, i);
                                let is_one = bit._eq(&z3::ast::BV::from_u64(ctx, 1, 1));
                                let count_at_i =
                                    z3::ast::BV::from_u64(ctx, ((bits - 1) - i) as u64, bits);
                                acc = is_one.ite(&count_at_i, &acc);
                            }
                            Ok(total_unary(acc, &val))
                        }
                        Clif::Ctz([ty, a]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let val = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let mut acc = z3::ast::BV::from_u64(ctx, bits as u64, bits);
                            for i in (0..bits).rev() {
                                let bit = val.value.extract(i, i);
                                let is_one = bit._eq(&z3::ast::BV::from_u64(ctx, 1, 1));
                                let count_at_i = z3::ast::BV::from_u64(ctx, i as u64, bits);
                                acc = is_one.ite(&count_at_i, &acc);
                            }
                            Ok(total_unary(acc, &val))
                        }
                        Clif::Cls([ty, a]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let val = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let sign_bit = val.value.extract(bits - 1, bits - 1);
                            let mut acc = z3::ast::BV::from_u64(ctx, (bits - 1) as u64, bits);
                            for i in 0..(bits - 1) {
                                let bit = val.value.extract(i, i);
                                let matches_sign = bit._eq(&sign_bit);
                                let count_at_i =
                                    z3::ast::BV::from_u64(ctx, ((bits - 1) - 1 - i) as u64, bits);
                                acc = matches_sign.ite(&acc, &count_at_i);
                            }
                            Ok(total_unary(acc, &val))
                        }
                        Clif::PopCnt([ty, a]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let val = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let mut sum = z3::ast::BV::from_u64(ctx, 0, bits);
                            for i in 0..bits {
                                let bit = val.value.extract(i, i).zero_ext(bits - 1);
                                sum = sum.bvadd(&bit);
                            }
                            Ok(total_unary(sum, &val))
                        }
                        Clif::Select([ty, c, a, b]) => {
                            let bits = type_bits(&expr[*ty], types)?;
                            check_expected(bits, expected)?;
                            let c = eval_node(
                                ctx,
                                expr,
                                *c,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let b = eval_node(
                                ctx,
                                expr,
                                *b,
                                Some(bits),
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let zero = z3::ast::BV::from_u64(ctx, 0, bits);
                            let cond = c.value._eq(&zero).not();
                            let branch_defined = cond.ite(&a.defined, &b.defined);
                            Ok(Z3Value {
                                value: cond.ite(&a.value, &b.value),
                                defined: z3_and(ctx, &[&c.defined, &branch_defined]),
                            })
                        }
                        Clif::Uextend([ty, a])
                        | Clif::Sextend([ty, a])
                        | Clif::Ireduce([ty, a]) => {
                            let out_bits = type_bits(&expr[*ty], types)?;
                            check_expected(out_bits, expected)?;
                            let a = eval_node(
                                ctx,
                                expr,
                                *a,
                                None,
                                types,
                                var_widths,
                                literal_bits,
                                seen_vars,
                            )?;
                            let in_bits = value_width(&a.value);
                            match node {
                                Clif::Uextend(_) if out_bits > in_bits => {
                                    Ok(total_unary(a.value.zero_ext(out_bits - in_bits), &a))
                                }
                                Clif::Sextend(_) if out_bits > in_bits => {
                                    Ok(total_unary(a.value.sign_ext(out_bits - in_bits), &a))
                                }
                                Clif::Ireduce(_) if out_bits < in_bits => {
                                    Ok(total_unary(a.value.extract(out_bits - 1, 0), &a))
                                }
                                _ => Err(()),
                            }
                        }
                    }
                }

                fn collect_type_vars(expr: &egg::RecExpr<Clif>, vars: &mut Vec<&'static str>) {
                    for node in expr.as_ref() {
                        let var = match node {
                            Clif::Ty => Some("ty"),
                            Clif::Ty1 => Some("ty1"),
                            Clif::Ty2 => Some("ty2"),
                            _ => None,
                        };
                        if let Some(var) = var {
                            if !vars.contains(&var) {
                                vars.push(var);
                            }
                        }
                    }
                }

                fn collect_bare_conversion_leaves(
                    expr: &egg::RecExpr<Clif>,
                    vars: &mut Vec<String>,
                    has_lit: &mut bool,
                ) {
                    for node in expr.as_ref() {
                        if let Clif::Uextend([_, a])
                        | Clif::Sextend([_, a])
                        | Clif::Ireduce([_, a]) = node
                        {
                            match &expr[*a] {
                                Clif::Var(v) => {
                                    let name = v.to_string();
                                    if !vars.contains(&name) {
                                        vars.push(name);
                                    }
                                }
                                Clif::Lit(_) => *has_lit = true,
                                _ => {}
                            }
                        }
                    }
                }

                fn explicit_output_bits(
                    expr: &egg::RecExpr<Clif>,
                    id: Id,
                    types: &HashMap<&'static str, u32>,
                ) -> Result<Option<u32>, ()> {
                    let type_id = match &expr[id] {
                        Clif::Iadd([ty, ..])
                        | Clif::Isub([ty, ..])
                        | Clif::Imul([ty, ..])
                        | Clif::Udiv([ty, ..])
                        | Clif::Sdiv([ty, ..])
                        | Clif::Urem([ty, ..])
                        | Clif::Srem([ty, ..])
                        | Clif::Umin([ty, ..])
                        | Clif::Umax([ty, ..])
                        | Clif::Smin([ty, ..])
                        | Clif::Smax([ty, ..])
                        | Clif::Band([ty, ..])
                        | Clif::Bor([ty, ..])
                        | Clif::Bxor([ty, ..])
                        | Clif::Ishl([ty, ..])
                        | Clif::Ushr([ty, ..])
                        | Clif::Sshr([ty, ..])
                        | Clif::Rotl([ty, ..])
                        | Clif::Rotr([ty, ..])
                        | Clif::Eq([ty, ..])
                        | Clif::Ne([ty, ..])
                        | Clif::Ule([ty, ..])
                        | Clif::Ult([ty, ..])
                        | Clif::Uge([ty, ..])
                        | Clif::Ugt([ty, ..])
                        | Clif::Sle([ty, ..])
                        | Clif::Slt([ty, ..])
                        | Clif::Sge([ty, ..])
                        | Clif::Sgt([ty, ..])
                        | Clif::Select([ty, ..]) => Some(*ty),
                        Clif::Ineg([ty, ..])
                        | Clif::Iabs([ty, ..])
                        | Clif::Bnot([ty, ..])
                        | Clif::Clz([ty, ..])
                        | Clif::Ctz([ty, ..])
                        | Clif::Cls([ty, ..])
                        | Clif::PopCnt([ty, ..])
                        | Clif::Uextend([ty, ..])
                        | Clif::Sextend([ty, ..])
                        | Clif::Ireduce([ty, ..]) => Some(*ty),
                        Clif::Lit(_) | Clif::Var(_) => None,
                        Clif::Ty | Clif::Ty1 | Clif::Ty2 => return Err(()),
                    };
                    type_id
                        .map(|ty| type_bits(&expr[ty], types).map(Some))
                        .unwrap_or(Ok(None))
                }

                fn extend_type_assignments(
                    assignments: Vec<HashMap<&'static str, u32>>,
                    var: &'static str,
                ) -> Vec<HashMap<&'static str, u32>> {
                    let mut out = vec![];
                    for assignment in assignments {
                        for bits in [8, 16, 32, 64] {
                            let mut assignment = assignment.clone();
                            assignment.insert(var, bits);
                            out.push(assignment);
                        }
                    }
                    out
                }

                fn extend_var_assignments(
                    assignments: Vec<HashMap<String, u32>>,
                    var: &str,
                ) -> Vec<HashMap<String, u32>> {
                    let mut out = vec![];
                    for assignment in assignments {
                        for bits in [8, 16, 32, 64] {
                            let mut assignment = assignment.clone();
                            assignment.insert(var.to_string(), bits);
                            out.push(assignment);
                        }
                    }
                    out
                }

                fn validate_assignment(
                    lhs: &egg::RecExpr<Clif>,
                    rhs: &egg::RecExpr<Clif>,
                    types: &HashMap<&'static str, u32>,
                    var_widths: &HashMap<String, u32>,
                    literal_bits: u32,
                ) -> Option<ValidationResult> {
                    let mut cfg = z3::Config::new();
                    cfg.set_timeout_msec(1000);
                    let ctx = z3::Context::new(&cfg);
                    let solver = z3::Solver::new(&ctx);
                    let lhs_root = Id::from(lhs.as_ref().len() - 1);
                    let rhs_root = Id::from(rhs.as_ref().len() - 1);
                    let lhs_bits = match explicit_output_bits(lhs, lhs_root, types) {
                        Ok(bits) => bits,
                        Err(_) => return None,
                    };
                    let rhs_bits = match explicit_output_bits(rhs, rhs_root, types) {
                        Ok(bits) => bits,
                        Err(_) => return None,
                    };
                    let lhs_expected = if lhs_bits.is_none() { rhs_bits } else { None };
                    let rhs_expected = if rhs_bits.is_none() { lhs_bits } else { None };
                    let mut lhs_vars = HashMap::default();
                    let mut rhs_vars = HashMap::default();
                    let lexpr = eval_node(
                        &ctx,
                        lhs,
                        lhs_root,
                        lhs_expected,
                        types,
                        var_widths,
                        literal_bits,
                        &mut lhs_vars,
                    );
                    let rexpr = eval_node(
                        &ctx,
                        rhs,
                        rhs_root,
                        rhs_expected,
                        types,
                        var_widths,
                        literal_bits,
                        &mut rhs_vars,
                    );
                    let (lexpr, rexpr) = match (lexpr, rexpr) {
                        (Ok(lexpr), Ok(rexpr)) => (lexpr, rexpr),
                        (Err(_), _) | (_, Err(_)) => return None,
                    };
                    if value_width(&lexpr.value) != value_width(&rexpr.value) {
                        return Some(ValidationResult::Invalid);
                    }
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
                    Some(match solver.check() {
                        SatResult::Sat => ValidationResult::Invalid,
                        SatResult::Unsat => ValidationResult::Valid,
                        SatResult::Unknown => ValidationResult::Unknown,
                    })
                }

                let lhs = Self::instantiate(lhs);
                let rhs = Self::instantiate(rhs);
                let mut type_vars = vec![];
                collect_type_vars(&lhs, &mut type_vars);
                collect_type_vars(&rhs, &mut type_vars);
                if type_vars.is_empty() {
                    type_vars.push("ty");
                }

                let mut type_assignments = vec![HashMap::default()];
                for var in type_vars {
                    type_assignments = extend_type_assignments(type_assignments, var);
                }

                let mut bare_vars = vec![];
                let mut has_bare_lit = false;
                collect_bare_conversion_leaves(&lhs, &mut bare_vars, &mut has_bare_lit);
                collect_bare_conversion_leaves(&rhs, &mut bare_vars, &mut has_bare_lit);

                let mut var_assignments = vec![HashMap::default()];
                for var in &bare_vars {
                    var_assignments = extend_var_assignments(var_assignments, var);
                }

                let literal_widths: &[u32] = if has_bare_lit {
                    &[8, 16, 32, 64]
                } else {
                    &[32]
                };

                let mut saw_legal = false;
                let mut saw_unknown = false;
                for types in &type_assignments {
                    for var_widths in &var_assignments {
                        for literal_bits in literal_widths {
                            match validate_assignment(&lhs, &rhs, types, var_widths, *literal_bits)
                            {
                                Some(ValidationResult::Valid) => saw_legal = true,
                                Some(ValidationResult::Invalid) => {
                                    return ValidationResult::Invalid
                                }
                                Some(ValidationResult::Unknown) => {
                                    saw_legal = true;
                                    saw_unknown = true;
                                }
                                None => {}
                            }
                        }
                    }
                }
                if !saw_legal {
                    ValidationResult::Invalid
                } else if saw_unknown {
                    ValidationResult::Unknown
                } else {
                    ValidationResult::Valid
                }
            }
        }
    };
}
