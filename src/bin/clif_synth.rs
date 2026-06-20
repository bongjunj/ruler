use std::str::FromStr;

use clap::Parser;
use log::info;
use ruler::{
    enumo::{Filter, Metric, Rule, Ruleset, Scheduler, Workload},
    recipe_utils::{recursive_rules, run_workload, Lang},
    Limits,
};

ruler::impl_clif_bv!(32);

#[derive(Parser)]
struct Args {
    #[clap(long, default_value_t = 4)]
    atoms: usize,

    #[clap(long, default_value_t = Strategy::Naive)]
    strategy: Strategy,
}

#[derive(Debug, Parser)]
enum Strategy {
    /// Naive Enumerative Synthesis
    Naive,

    /// Strategies shown on the halide case study
    Halide,
}

impl FromStr for Strategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "naive" => Ok(Self::Naive),
            "halide" => Ok(Self::Halide),
            _ => Err("unknown strategy {s}".to_string()),
        }
    }
}

impl ToString for Strategy {
    fn to_string(&self) -> String {
        match self {
            Strategy::Naive => "naive".to_string(),
            Strategy::Halide => "halide".to_string(),
        }
    }
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    match args.strategy {
        Strategy::Naive => naive_synthesis(args.atoms),
        Strategy::Halide => halide_like_synthesis(args.atoms),
    }
}

fn naive_synthesis(atoms: usize) {
    let rules = recursive_rules(
        Metric::Atoms,
        atoms,
        Lang::new(
            &["0", "1", "-1", "2", "31", "32", "33"],
            &["x", "y", "z"],
            &[
                &["ineg", "iabs", "bnot", "ctz", "clz", "cls", "popcnt"],
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

    let vars = Workload::new(["a", "b", "c"]);
    let consts = Workload::new(["C1", "C2", "C3"]);

    let unops = Workload::new(["ineg", "iabs", "bnot", "clz", "ctz", "cls", "popcnt"]);
    let cf_unops = Workload::new([
        "cf-ineg", "cf-iabs", "cf-bnot", "cf-clz", "cf-ctz", "cf-cls",
    ]);
    let binops = Workload::new([
        "iadd", "isub", "imul", "udiv", "sdiv", "urem", "srem", "band", "bor", "bxor", "ishl",
        "ushr", "sshr", "rotl", "rotr", "umin", "umax", "smin", "smax", "eq", "ne", "ule", "ult",
        "uge", "ugt", "sle", "slt", "sge", "sgt",
    ]);
    let cf_binops = Workload::new([
        "cf-iadd", "cf-isub", "cf-imul", "cf-udiv", "cf-sdiv", "cf-urem", "cf-srem", "cf-band",
        "cf-bor", "cf-bxor", "cf-ishl", "cf-ushr", "cf-sshr", "cf-rotl", "cf-rotr", "cf-umin",
        "cf-umax", "cf-smin", "cf-smax", "cf-eq", "cf-ne", "cf-ule", "cf-ult", "cf-uge", "cf-ugt",
        "cf-sle", "cf-slt", "cf-sge", "sgcf-t",
    ]);

    let symbol = vars.clone().append(consts.clone());

    let lhs = Workload::new(["(OP1 V)", "(OP2 V V)", "(select V V V)"])
        .plug("V", &symbol)
        .plug("OP1", &unops)
        .plug("OP2", &binops);

    let rhs = Workload::new(["(OP1 V)", "(OP2 V V)", "(select V V V)"])
        .plug("V", &vars)
        .plug("OP1", &unops.clone().append(cf_unops.clone()))
        .plug("OP2", &binops.clone().append(cf_binops.clone()));

    let rules: Ruleset<Clif> = Ruleset::default();
    // NOTE: Enumo does not support type terms. It builds a single e-graph
    // from a set of terms, and extract equivalent e-classes.
    // It means that it isn't easy to put constraints on LHS/RHS separately.
    // For example, we want to constraint <op1> and <op2> in "cf-iadd <op1> <op2>"
    // to expand to constant folding operations and C1/C2/C3.
    // However such constraint need type-system. However, this will need
    // much more work.
    todo!()
}

fn halide_like_synthesis(atoms: usize) {
    let mut all_rules = Ruleset::<Clif>::default();

    let arith_bits = recursive_rules(
        Metric::Atoms,
        atoms,
        Lang::new(
            &["0", "1", "-1", "2"],
            &["x", "y", "z"],
            &[
                &["ineg", "iabs", "bnot"],
                &[
                    "iadd", "isub", "imul", "band", "bor", "bxor", "umin", "umax", "smin", "smax",
                ],
            ],
        ),
        all_rules.clone(),
    );
    all_rules.extend(arith_bits.clone());

    info!("arith bits rules are synthesized");

    let shift_bits = recursive_rules(
        Metric::Atoms,
        atoms,
        Lang::new(
            &["0", "1", "-1", "2", "31", "32", "33"],
            &["x", "y", "z"],
            &[
                &["bnot"],
                &[
                    "ishl", "ushr", "sshr", "rotl", "rotr", "band", "bor", "bxor",
                ],
            ],
        ),
        all_rules.clone(),
    );
    all_rules.extend(shift_bits);

    info!("shift/rot rules are synthesized");

    let cmp_select = recursive_rules(
        Metric::Atoms,
        atoms,
        Lang::new(
            &["0", "1", "-1", "2"],
            &["x", "y", "z"],
            &[
                &["ineg", "iabs", "bnot"],
                &[
                    "eq", "ne", "ule", "ult", "uge", "ugt", "sle", "slt", "sge", "sgt", "umin",
                    "umax", "smin", "smax",
                ],
                &["select"],
            ],
        ),
        all_rules.clone(),
    );
    all_rules.extend(cmp_select);

    info!("cmp/select rules are synthesized");

    let div_rem = recursive_rules(
        Metric::Atoms,
        atoms,
        Lang::new(
            &["0", "1", "-1", "2"],
            &["x", "y", "z"],
            &[&["ineg", "iabs"], &["udiv", "sdiv", "urem", "srem"]],
        ),
        all_rules.clone(),
    );
    all_rules.extend(div_rem);
    info!("div/rem rules are synthesized");

    let full_atoms = atoms.saturating_sub(1).max(1);
    let full = recursive_rules(
        Metric::Atoms,
        full_atoms,
        Lang::new(
            &["0", "1", "-1", "2", "31", "32", "33"],
            &["x", "y", "z"],
            &[
                &["ineg", "iabs", "bnot", "ctz", "clz", "cls", "popcnt"],
                &[
                    "iadd", "isub", "imul", "udiv", "sdiv", "urem", "srem", "band", "bor", "bxor",
                    "ishl", "ushr", "sshr", "rotl", "rotr", "umin", "umax", "smin", "smax", "eq",
                    "ne", "ule", "ult", "uge", "ugt", "sle", "slt", "sge", "sgt",
                ],
                &["select"],
            ],
        ),
        all_rules.clone(),
    );
    all_rules.extend(full);

    let workload_limits = Limits {
        iter: 1,
        node: 100_000,
        match_: 100_000,
    };

    let nested_bops_arith = Workload::new(&["(bop e e)", "v"])
        .plug("e", &Workload::new(&["(bop v v)", "(uop v)", "v"]))
        .plug("bop", &Workload::new(&["iadd", "isub", "imul"]))
        .plug("uop", &Workload::new(&["ineg", "bnot"]))
        .plug("v", &Workload::new(&["x", "y", "z"]))
        .filter(Filter::Canon(vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
        ]));
    let new = run_workload(
        nested_bops_arith,
        all_rules.clone(),
        workload_limits,
        workload_limits,
        true,
    );
    all_rules.extend(new);

    let nested_bops_full = Workload::new(&["(bop e e)", "v"])
        .plug("e", &Workload::new(&["(bop v v)", "(uop v)", "v"]))
        .plug("bop", &Workload::new(&["eq", "ne", "ule", "ult"]))
        .plug("uop", &Workload::new(&["ineg", "bnot"]))
        .plug("v", &Workload::new(&["x", "y", "z"]))
        .filter(Filter::Canon(vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
        ]));
    let new = run_workload(
        nested_bops_full,
        all_rules.clone(),
        workload_limits,
        workload_limits,
        true,
    );
    all_rules.extend(new);

    let select_base = Workload::new([
        "(select V V V)",
        "(select V (OP V V) V)",
        "(select V V (OP V V))",
        "(select V (OP V V) (OP V V))",
        "(OP V (select V V V))",
    ])
    .plug("V", &Workload::new(["x", "y", "z", "w"]));

    let arith_select = select_base.clone().plug(
        "OP",
        &Workload::new(["iadd", "isub", "imul", "band", "bor", "bxor"]),
    );
    let new = run_workload(
        arith_select,
        arith_bits.clone(),
        Limits::synthesis(),
        workload_limits,
        true,
    );
    all_rules.extend(new);

    info!("select-plugged rules are synthesized");

    let minmax_select = select_base.plug("OP", &Workload::new(["umin", "umax", "smin", "smax"]));
    let new = run_workload(
        minmax_select,
        arith_bits,
        Limits::synthesis(),
        workload_limits,
        true,
    );
    all_rules.extend(new);

    all_rules.pretty_print();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_folds_include_iadd_schema() {
        let rules = synthesize_const_fold_rules().to_str_vec();

        assert!(rules.contains(&"(iadd (k ?a) (k ?b)) ==> (cf-iadd ?a ?b)".to_string()));
    }

    #[test]
    fn constant_fold_workload_keeps_markers_on_their_sides() {
        let terms: Vec<_> = const_fold_workload()
            .force()
            .into_iter()
            .map(|term| term.to_string())
            .collect();

        let terms: Vec<_> = terms
            .into_iter()
            .map(|term| term.replace(' ', ""))
            .collect();

        assert_eq!(terms, vec!["(iadd(ka)(kb))", "(cf-iaddab)"]);
    }

    #[test]
    fn constant_folds_filter_reverse_rule() {
        let rules = synthesize_const_fold_rules().to_str_vec();

        assert!(!rules.iter().any(|rule| rule.starts_with("(cf-iadd ?")));
    }
}
