use clap::Parser;
use ruler::{
    enumo::{Metric, Ruleset},
    recipe_utils::{recursive_rules, Lang},
};

ruler::impl_clif_bv!(32);

#[derive(Parser)]
struct Args {
    #[clap(long, default_value_t = 4)]
    atoms: usize,
}

fn main() {
    let args = Args::parse();
    let rules = recursive_rules(
        Metric::Atoms,
        args.atoms,
        Lang::new(
            &["0", "1", "-1", "2", "31", "32", "33"],
            &["x", "y", "z"],
            &[
                &["ineg", "bnot", "ctz", "clz", "cls", "popcnt"],
                &[
                    "iadd", "isub", "imul", "udiv", "sdiv", "urem", "srem", "band", "bor", "bxor",
                    "ishl", "ushr", "sshr", "rotl", "rotr", "umin", "umax", "smin", "smax", "eq",
                    "ne", "ule", "ult", "uge", "ugt", "sle", "slt", "sge", "sgt",
                ],
                &["select"],
            ],
        ),
        Ruleset::<Clif>::default(),
    );

    rules.pretty_print();
}
