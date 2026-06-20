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
