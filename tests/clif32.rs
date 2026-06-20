use ruler::{
    enumo::{Metric, Rule, Ruleset},
    recipe_utils::{recursive_rules, Lang},
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
fn z3_validates_rotation_semantics() {
    assert!(valid("(rotl a 0) <=> a"));
    assert!(valid("(rotr a 0) <=> a"));
    assert!(valid("(rotl a 32) <=> a"));
    assert!(valid("(rotr a 32) <=> a"));
    assert!(valid("(rotl a 33) <=> (rotl a 1)"));
    assert!(valid("(rotr a 33) <=> (rotr a 1)"));
    assert!(valid("(rotr (rotl a b) b) <=> a"));
    assert!(valid("(rotl (rotr a b) b) <=> a"));
    assert!(valid("(rotl a 1) <=> (bor (ishl a 1) (ushr a 31))"));
    assert!(valid("(rotr a 1) <=> (bor (ushr a 1) (ishl a 31))"));
}

#[test]
fn z3_validates_bits_semantics() {
    assert!(valid("(clz 0) <=> 32"));
    assert!(valid("(ctz 0) <=> 32"));
    assert!(valid("(cls 0) <=> 31"));
    assert!(valid("(popcnt 0) <=> 0"));

    assert!(valid("(clz -1) <=> 0"));
    assert!(valid("(ctz -1) <=> 0"));
    assert!(valid("(cls -1) <=> 31"));
    assert!(valid("(popcnt -1) <=> 32"));

    assert!(valid("(clz 2147483648) <=> 0"));
    assert!(valid("(ctz 2147483648) <=> 31"));
    assert!(valid("(cls 2147483648) <=> 0"));
    assert!(valid("(popcnt 2147483648) <=> 1"));

    assert!(valid("(clz 15728640) <=> 8"));
    assert!(valid("(ctz 15728640) <=> 20"));
    assert!(valid("(cls 15728640) <=> 7"));
    assert!(valid("(popcnt 15728640) <=> 4"));

    assert!(valid("(popcnt a) <=> (popcnt (rotl a 1))"));
    assert!(valid("(popcnt a) <=> (popcnt (rotr a 1))"));
    assert!(valid("(popcnt a) <=> (popcnt (rotl a 32))"));
    assert!(valid("(popcnt a) <=> (popcnt (rotr a 32))"));
}

#[test]
fn z3_validates_masked_shift_identities() {
    assert!(valid("(sshr a 0) <=> a"));
    assert!(valid("(sshr a 32) <=> a"));
    assert!(valid("(sshr a 33) <=> (sshr a 1)"));
    assert!(valid("(ushr a 32) <=> a"));
    assert!(valid("(ishl a 32) <=> a"));
}

#[test]
fn parses_negative_constants_as_wrapped_bitvectors() {
    assert_eq!("-1".parse::<BV>().unwrap().0, 0xffff_ffff);
}

#[test]
#[ignore = "can take a long time..."]
fn synthesize_clif32_shift_rules() {
    let rules = recursive_rules(
        Metric::Atoms,
        5,
        Lang::new(
            &["0", "1", "-1", "31", "32", "33"],
            &["a", "b"],
            &[
                &["ineg", "bnot"],
                &[
                    "iadd", "isub", "imul", "band", "bor", "bxor", "ishl", "ushr", "sshr",
                ],
            ],
        ),
        Ruleset::<Clif>::default(),
    );

    rules.pretty_print();
}
