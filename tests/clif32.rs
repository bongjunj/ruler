use ruler::{
    enumo::{Metric, Rule, Ruleset},
    recipe_utils::{recursive_rules_with_base, Lang},
};

ruler::impl_clif_bv!(32);

fn valid(rule: &str) -> bool {
    let (forward, backward) = Rule::<Clif>::from_string(rule).unwrap();
    forward.is_valid() && backward.map_or(true, |rule| rule.is_valid())
}

#[test]
fn concrete_shift_semantics_mask_amounts() {
    let sign_bit = BV::from(0x8000_0000u128);
    let all_ones = BV::from(0xffff_ffffu128);

    assert_eq!(sign_bit.wrapping_sshr(BV::from(1)).0, 0xc000_0000);
    assert_eq!(sign_bit.wrapping_sshr(BV::from(31)).0, all_ones.0);
    assert_eq!(sign_bit.wrapping_sshr(BV::from(32)).0, sign_bit.0);
    assert_eq!(
        sign_bit.wrapping_sshr(BV::from(33)).0,
        sign_bit.wrapping_sshr(BV::from(1)).0
    );

    let x = BV::from(0x8000_0001u128);
    assert_eq!(x.wrapping_ishl(BV::from(32)).0, x.0);
    assert_eq!(
        x.wrapping_ishl(BV::from(33)).0,
        x.wrapping_ishl(BV::from(1)).0
    );
    assert_eq!(x.wrapping_ushr(BV::from(32)).0, x.0);
    assert_eq!(
        x.wrapping_ushr(BV::from(33)).0,
        x.wrapping_ushr(BV::from(1)).0
    );
}

#[test]
fn rotation_semantics() {
    let x = BV::from(0x8000_0001u128);

    assert_eq!(x.wrapping_rotl(BV::from(0)).0, x.0);
    assert_eq!(x.wrapping_rotl(BV::from(1)).0, 0x0000_0003);
    assert_eq!(
        x.wrapping_rotl(BV::from(31)).0,
        x.wrapping_rotr(BV::from(1)).0
    );
    assert_eq!(x.wrapping_rotl(BV::from(32)).0, x.0);
    assert_eq!(
        x.wrapping_rotl(BV::from(33)).0,
        x.wrapping_rotl(BV::from(1)).0
    );

    assert_eq!(x.wrapping_rotr(BV::from(0)).0, x.0);
    assert_eq!(x.wrapping_rotr(BV::from(1)).0, 0xc000_0000);
    assert_eq!(
        x.wrapping_rotr(BV::from(31)).0,
        x.wrapping_rotl(BV::from(1)).0
    );
    assert_eq!(x.wrapping_rotr(BV::from(32)).0, x.0);
    assert_eq!(
        x.wrapping_rotr(BV::from(33)).0,
        x.wrapping_rotr(BV::from(1)).0
    );
}

#[test]
fn bits_semantics() {
    let zero = BV::from(0);
    assert_eq!(zero.count_leading_zeros().0, 32);
    assert_eq!(zero.count_trailing_zeros().0, 32);
    assert_eq!(zero.count_leading_signbits().0, 31);
    assert_eq!(zero.popcnt().0, 0);

    let all_ones = BV::from(0xffff_ffffu128);
    assert_eq!(all_ones.count_leading_zeros().0, 0);
    assert_eq!(all_ones.count_trailing_zeros().0, 0);
    assert_eq!(all_ones.count_leading_signbits().0, 31);
    assert_eq!(all_ones.popcnt().0, 32);

    let sign_bit = BV::from(0x8000_0000u128);
    assert_eq!(sign_bit.count_leading_zeros().0, 0);
    assert_eq!(sign_bit.count_trailing_zeros().0, 31);
    assert_eq!(sign_bit.count_leading_signbits().0, 0);
    assert_eq!(sign_bit.popcnt().0, 1);

    let sparse = BV::from(0x00f0_0000u128);
    assert_eq!(sparse.count_leading_zeros().0, 8);
    assert_eq!(sparse.count_trailing_zeros().0, 20);
    assert_eq!(sparse.count_leading_signbits().0, 7);
    assert_eq!(sparse.popcnt().0, 4);

    assert_eq!(BV::from(0xc000_0000u128).count_leading_signbits().0, 1);
    assert_eq!(BV::from(0x4000_0000u128).count_leading_signbits().0, 0);
}

#[test]
fn comparison_and_select_semantics() {
    let zero = BV::from(0);
    let one = BV::from(1);
    let two = BV::from(2);
    let neg_one = BV::from(0xffff_ffffu128);
    let sign_bit = BV::from(0x8000_0000u128);

    assert_eq!(zero.bv_eq(zero).0, 1);
    assert_eq!(zero.bv_eq(one).0, 0);
    assert_eq!(zero.bv_ne(one).0, 1);
    assert_eq!(one.bv_ne(one).0, 0);

    assert_eq!(zero.ult(neg_one).0, 1);
    assert_eq!(neg_one.ugt(zero).0, 1);
    assert_eq!(zero.ule(zero).0, 1);
    assert_eq!(neg_one.uge(zero).0, 1);

    assert_eq!(neg_one.slt(zero).0, 1);
    assert_eq!(zero.sgt(neg_one).0, 1);
    assert_eq!(sign_bit.slt(zero).0, 1);
    assert_eq!(neg_one.sle(neg_one).0, 1);
    assert_eq!(zero.sge(neg_one).0, 1);

    assert_eq!(zero.select(one, two).0, two.0);
    assert_eq!(one.select(one, two).0, one.0);
    assert_eq!(two.select(one, zero).0, one.0);
}

#[test]
fn iabs_semantics() {
    let zero = BV::from(0);
    let one = BV::from(1);
    let neg_one = BV::from(0xffff_ffffu128);
    let min = BV::from(0x8000_0000u128);

    assert_eq!(zero.iabs().0, zero.0);
    assert_eq!(one.iabs().0, one.0);
    assert_eq!(neg_one.iabs().0, one.0);
    assert_eq!(min.iabs().0, min.0);
}

#[test]
fn division_and_remainder_semantics() {
    let zero = BV::from(0);
    let one = BV::from(1);
    let two = BV::from(2);
    let three = BV::from(3);
    let six = BV::from(6);
    let neg_one = BV::from(0xffff_ffffu128);
    let neg_two = BV::from(0xffff_fffeu128);
    let neg_three = BV::from(0xffff_fffdu128);
    let min = BV::from(0x8000_0000u128);

    assert_eq!(six.checked_udiv(two).unwrap().0, 3);
    assert_eq!(six.checked_urem(BV::from(4)).unwrap().0, 2);
    assert_eq!(neg_two.checked_sdiv(two).unwrap().0, neg_one.0);
    assert_eq!(neg_three.checked_srem(two).unwrap().0, neg_one.0);

    assert_eq!(six.checked_udiv(zero), None);
    assert_eq!(six.checked_urem(zero), None);
    assert_eq!(six.checked_sdiv(zero), None);
    assert_eq!(six.checked_srem(zero), None);
    assert_eq!(min.checked_sdiv(neg_one), None);
    assert_eq!(min.checked_srem(neg_one), Some(zero));

    assert_eq!(three.checked_sdiv(one).unwrap().0, three.0);
    assert_eq!(three.checked_srem(one).unwrap().0, zero.0);
}

#[test]
fn min_and_max_semantics() {
    let zero = BV::from(0);
    let neg_one = BV::from(0xffff_ffffu128);
    let min = BV::from(0x8000_0000u128);

    assert_eq!(zero.umin(neg_one).0, zero.0);
    assert_eq!(zero.umax(neg_one).0, neg_one.0);
    assert_eq!(zero.smin(neg_one).0, neg_one.0);
    assert_eq!(zero.smax(neg_one).0, zero.0);
    assert_eq!(min.smin(zero).0, min.0);
    assert_eq!(min.smax(zero).0, zero.0);
}

#[test]
fn z3_validates_rotation_semantics() {
    assert!(valid("(rotl ty a 0) <=> a"));
    assert!(valid("(rotr ty a 0) <=> a"));
    assert!(valid("(rotr ty (rotl ty a 1) 1) <=> a"));
    assert!(valid("(rotl ty (rotr ty a 1) 1) <=> a"));
    assert!(!valid("(rotl ty a 32) <=> a"));
}

#[test]
fn z3_validates_bits_semantics() {
    assert!(valid("(popcnt ty 0) <=> 0"));
    assert!(valid("(clz ty -1) <=> 0"));
    assert!(valid("(ctz ty -1) <=> 0"));
    assert!(valid("(popcnt ty a) <=> (popcnt ty (rotl ty a 1))"));
    assert!(valid("(popcnt ty a) <=> (popcnt ty (rotr ty a 1))"));
    assert!(!valid("(clz ty 0) <=> 32"));
}

#[test]
fn z3_validates_masked_shift_identities() {
    assert!(valid("(sshr ty a 0) <=> a"));
    assert!(valid("(ushr ty a 0) <=> a"));
    assert!(valid("(ishl ty a 0) <=> a"));
    assert!(!valid("(sshr ty a 32) <=> a"));
}

#[test]
fn z3_validates_comparison_and_select_semantics() {
    assert!(valid("(ult ty a b) <=> (ugt ty b a)"));
    assert!(valid("(ule ty a b) <=> (uge ty b a)"));
    assert!(valid("(slt ty a b) <=> (sgt ty b a)"));
    assert!(valid("(sle ty a b) <=> (sge ty b a)"));
    assert!(valid("(eq ty a b) <=> (eq ty b a)"));
    assert!(valid("(ne ty a b) <=> (ne ty b a)"));

    assert!(valid("(select ty 0 a b) <=> b"));
    assert!(valid("(select ty 1 a b) <=> a"));
    assert!(valid("(select ty 2 a b) <=> a"));

    assert!(!valid("(ult ty -1 0) <=> 1"));
    assert!(!valid("(slt ty 0 -1) <=> 1"));
}

#[test]
fn z3_validates_iabs_semantics() {
    assert!(valid("(iabs ty 0) <=> 0"));
    assert!(valid("(iabs ty 1) <=> 1"));
    assert!(valid("(iabs ty -1) <=> 1"));
    assert!(valid("(iabs ty (iabs ty a)) <=> (iabs ty a)"));

    assert!(!valid("(iabs ty a) <=> a"));
}

#[test]
fn z3_validates_division_and_remainder_semantics() {
    assert!(valid("(udiv ty a 1) <=> a"));
    assert!(valid("(sdiv ty a 1) <=> a"));
    assert!(valid("(urem ty a 1) <=> 0"));
    assert!(valid("(srem ty a 1) <=> 0"));

    assert!(valid("(select ty 0 (udiv ty a 0) b) <=> b"));
    assert!(!valid("(select ty 1 (udiv ty a 0) b) <=> b"));

    assert!(!valid("(udiv ty a 0) <=> 0"));
    assert!(!valid("(udiv ty -2 2) <=> (sdiv ty -2 2)"));
}

#[test]
fn z3_validates_min_and_max_semantics() {
    assert!(valid("(umin ty a b) <=> (umin ty b a)"));
    assert!(valid("(umax ty a b) <=> (umax ty b a)"));
    assert!(valid("(smin ty a b) <=> (smin ty b a)"));
    assert!(valid("(smax ty a b) <=> (smax ty b a)"));

    assert!(valid("(umin ty a a) <=> a"));
    assert!(valid("(umax ty a a) <=> a"));
    assert!(valid("(smin ty a a) <=> a"));
    assert!(valid("(smax ty a a) <=> a"));

    assert!(!valid("(umin ty 0 -1) <=> -1"));
    assert!(!valid("(smin ty 0 -1) <=> 0"));
}

#[test]
fn z3_validates_bitwidth_polymorphic_rules_and_conversions() {
    assert!(valid("(iadd ty a b) <=> (iadd ty b a)"));
    assert!(valid("(rotl ty a 0) <=> a"));
    assert!(valid("(rotr ty a 0) <=> a"));
    assert!(!valid("(rotl ty a 32) <=> a"));
    assert!(!valid("(iadd ty a 255) <=> (isub ty a 1)"));

    assert!(valid(
        "(ireduce ty1 (uextend ty2 (iadd ty1 a 0))) <=> (iadd ty1 a 0)"
    ));
    assert!(valid(
        "(ireduce ty1 (sextend ty2 (iadd ty1 a 0))) <=> (iadd ty1 a 0)"
    ));
    assert!(valid("(eq ty2 (uextend ty2 0) 0) <=> 1"));
    assert!(!valid("(uextend ty a) <=> a"));
    assert!(!valid("(ireduce ty a) <=> a"));
}

#[test]
fn parses_negative_constants_as_wrapped_bitvectors() {
    assert_eq!("-1".parse::<BV>().unwrap().0, 0xffff_ffff);
}

#[test]
#[ignore = "can take a long time..."]
fn synthesize_clif32_shift_rules() {
    let rules = recursive_rules_with_base(
        Metric::Atoms,
        5,
        Lang::new(
            &["0", "1", "-1", "31", "32", "33"],
            &["a", "b"],
            &[
                &["ineg", "iabs", "bnot", "uextend", "sextend", "ireduce"],
                &[
                    "iadd", "isub", "imul", "udiv", "sdiv", "urem", "srem", "band", "bor", "bxor",
                    "ishl", "ushr", "sshr", "umin", "umax", "smin", "smax", "eq", "ne", "ule",
                    "ult", "uge", "ugt", "sle", "slt", "sge", "sgt",
                ],
                &["select"],
            ],
        ),
        ruler::enumo::Workload::new([
            "VAR",
            "VAL",
            "(OP1 ty EXPR)",
            "(OP1 ty1 EXPR)",
            "(OP1 ty2 EXPR)",
            "(OP2 ty EXPR EXPR)",
            "(OP2 ty1 EXPR EXPR)",
            "(OP2 ty2 EXPR EXPR)",
            "(OP3 ty EXPR EXPR EXPR)",
            "(OP3 ty1 EXPR EXPR EXPR)",
            "(OP3 ty2 EXPR EXPR EXPR)",
        ]),
        Ruleset::<Clif>::default(),
    );

    rules.pretty_print();
}
