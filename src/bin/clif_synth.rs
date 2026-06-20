use std::str::FromStr;

use clap::Parser;
use log::info;
use ruler::{
    enumo::{Filter, Metric, Ruleset, Workload},
    recipe_utils::{recursive_rules, run_workload, Lang},
    Limits, SynthLanguage,
};

#[derive(Parser)]
struct Args {
    #[clap(long, default_value_t = 4)]
    atoms: usize,

    #[clap(long, default_value_t = 32)]
    bits: u16,

    #[clap(long, default_value_t = Strategy::Naive)]
    strategy: Strategy,
}

#[derive(Clone, Copy, Debug, Parser)]
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

    match args.bits {
        8 => clif8::run(args.strategy, args.atoms),
        16 => clif16::run(args.strategy, args.atoms),
        32 => clif32::run(args.strategy, args.atoms),
        64 => clif64::run(args.strategy, args.atoms),
        bits => panic!(
            "unsupported CLIF bit width {}; expected one of 8, 16, 32, or 64",
            bits
        ),
    }
}

mod clif8 {
    ruler::impl_clif_bv!(8);

    pub fn run(strategy: super::Strategy, atoms: usize) {
        super::run::<Clif>(strategy, atoms, 8);
    }
}

mod clif16 {
    ruler::impl_clif_bv!(16);

    pub fn run(strategy: super::Strategy, atoms: usize) {
        super::run::<Clif>(strategy, atoms, 16);
    }
}

mod clif32 {
    ruler::impl_clif_bv!(32);

    pub fn run(strategy: super::Strategy, atoms: usize) {
        super::run::<Clif>(strategy, atoms, 32);
    }
}

mod clif64 {
    ruler::impl_clif_bv!(64);

    pub fn run(strategy: super::Strategy, atoms: usize) {
        super::run::<Clif>(strategy, atoms, 64);
    }
}

fn run<L: SynthLanguage>(strategy: Strategy, atoms: usize, bits: u16) {
    match strategy {
        Strategy::Naive => naive_synthesis::<L>(atoms, bits),
        Strategy::Halide => halide_like_synthesis::<L>(atoms, bits),
    }
}

fn lang(vals: Vec<String>, vars: &[&str], ops: &[&[&str]]) -> Lang {
    Lang {
        vals,
        vars: vars.iter().map(|v| v.to_string()).collect(),
        ops: ops
            .iter()
            .map(|ops| ops.iter().map(|op| op.to_string()).collect())
            .collect(),
    }
}

fn base_vals() -> Vec<String> {
    ["0", "1", "-1", "2"]
        .iter()
        .map(|v| v.to_string())
        .collect()
}

fn shift_vals(bits: u16) -> Vec<String> {
    let mut vals = base_vals();
    vals.extend([
        (bits - 1).to_string(),
        bits.to_string(),
        (bits + 1).to_string(),
    ]);
    vals.sort();
    vals.dedup();
    vals
}

fn naive_synthesis<L: SynthLanguage>(atoms: usize, bits: u16) {
    let rules = recursive_rules(
        Metric::Atoms,
        atoms,
        lang(
            shift_vals(bits),
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
        Ruleset::<L>::default(),
    );

    rules.pretty_print();
}

fn halide_like_synthesis<L: SynthLanguage>(atoms: usize, bits: u16) {
    let mut all_rules = Ruleset::<L>::default();

    let arith_bits = recursive_rules(
        Metric::Atoms,
        atoms,
        lang(
            base_vals(),
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
        lang(
            shift_vals(bits),
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
        lang(
            base_vals(),
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
        lang(
            base_vals(),
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
        lang(
            shift_vals(bits),
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
