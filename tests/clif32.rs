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
