use std::{fs, path::PathBuf, process, str::FromStr};

use clap::Parser;
use log::info;
use ruler::{
    enumo::{Filter, Metric, Rule, Ruleset, Workload},
    recipe_utils::{recursive_rules_with_base_allow_empty_and_canon, run_workload, Lang},
    Limits, SynthLanguage,
};

#[derive(Parser)]
struct Args {
    #[clap(long, default_value_t = 4)]
    atoms: usize,

    #[clap(long, default_value_t = Strategy::Naive)]
    strategy: Strategy,

    #[clap(long)]
    prior: Option<PathBuf>,
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
    // env_logger::init();

    let args = Args::parse();
    if let Err(err) = clif::run(args.strategy, args.atoms, args.prior) {
        eprintln!("{err}");
        process::exit(1);
    }
}

mod clif {
    // Concrete sampling width for cvec generation only. Rule validation is
    // bitwidth-polymorphic over the symbolic CLIF type atoms.
    ruler::impl_clif_bv!(32);

    pub fn run(
        strategy: super::Strategy,
        atoms: usize,
        prior: Option<std::path::PathBuf>,
    ) -> Result<(), String> {
        super::run::<Clif>(strategy, atoms, prior)
    }
}

fn run<L: SynthLanguage>(
    strategy: Strategy,
    atoms: usize,
    prior: Option<PathBuf>,
) -> Result<(), String> {
    let prior = load_prior::<L>(prior)?;
    match strategy {
        Strategy::Naive => naive_synthesis::<L>(atoms, prior),
        Strategy::Halide => halide_like_synthesis::<L>(atoms, prior),
    }
    Ok(())
}

fn load_prior<L: SynthLanguage>(path: Option<PathBuf>) -> Result<Ruleset<L>, String> {
    let Some(path) = path else {
        return Ok(Ruleset::default());
    };
    let input = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read prior rules {}: {err}", path.display()))?;
    let mut rules = Ruleset::default();
    for (line_no, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let (forward, backward) = Rule::<L>::from_string(line)
            .map_err(|err| format!("{}:{}: {err}", path.display(), line_no + 1))?;
        if !forward.is_valid() {
            return Err(format!(
                "{}:{}: invalid prior rule `{}`",
                path.display(),
                line_no + 1,
                forward
            ));
        }
        rules.add(forward);
        if let Some(backward) = backward {
            if !backward.is_valid() {
                return Err(format!(
                    "{}:{}: invalid prior rule `{}`",
                    path.display(),
                    line_no + 1,
                    backward
                ));
            }
            rules.add(backward);
        }
    }
    Ok(rules)
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

fn clif_base_lang() -> Workload {
    Workload::new([
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
    ])
}

fn clif_rules<L: SynthLanguage>(
    metric: Metric,
    atoms: usize,
    lang: Lang,
    prior: Ruleset<L>,
) -> Ruleset<L> {
    let canon_symbols = vec![
        lang.vars.clone(),
        vec!["ty".to_string(), "ty1".to_string(), "ty2".to_string()],
    ];
    recursive_rules_with_base_allow_empty_and_canon(
        metric,
        atoms,
        lang,
        clif_base_lang(),
        prior,
        atoms + 1,
        canon_symbols,
    )
}

fn naive_synthesis<L: SynthLanguage>(atoms: usize, prior: Ruleset<L>) {
    let rules = clif_rules(
        Metric::Atoms,
        atoms,
        lang(
            base_vals(),
            &["x", "y", "z"],
            &[
                &[
                    "ineg", "iabs", "bnot", "ctz", "clz", "cls", "popcnt", "uextend", "sextend",
                    "ireduce",
                ],
                &[
                    "iadd", "isub", "imul", "udiv", "sdiv", "urem", "srem", "band", "bor", "bxor",
                    "ishl", "ushr", "sshr", "rotl", "rotr", "umin", "umax", "smin", "smax", "eq",
                    "ne", "ule", "ult", "uge", "ugt", "sle", "slt", "sge", "sgt",
                ],
                &["select"],
            ],
        ),
        prior,
    );

    rules.pretty_print();
}

fn halide_like_synthesis<L: SynthLanguage>(atoms: usize, prior: Ruleset<L>) {
    let mut all_rules = prior;

    let arith_bits = clif_rules(
        Metric::Atoms,
        atoms,
        lang(
            base_vals(),
            &["x", "y", "z"],
            &[
                &["ineg", "iabs", "bnot", "uextend", "sextend", "ireduce"],
                &[
                    "iadd", "isub", "imul", "band", "bor", "bxor", "umin", "umax", "smin", "smax",
                ],
            ],
        ),
        all_rules.clone(),
    );
    all_rules.extend(arith_bits.clone());

    info!("arith bits rules are synthesized");

    let shift_bits = clif_rules(
        Metric::Atoms,
        atoms,
        lang(
            base_vals(),
            &["x", "y", "z"],
            &[
                &["bnot", "uextend", "sextend", "ireduce"],
                &[
                    "ishl", "ushr", "sshr", "rotl", "rotr", "band", "bor", "bxor",
                ],
            ],
        ),
        all_rules.clone(),
    );
    all_rules.extend(shift_bits);

    info!("shift/rot rules are synthesized");

    let cmp_select = clif_rules(
        Metric::Atoms,
        atoms,
        lang(
            base_vals(),
            &["x", "y", "z"],
            &[
                &["ineg", "iabs", "bnot", "uextend", "sextend", "ireduce"],
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

    let div_rem = clif_rules(
        Metric::Atoms,
        atoms,
        lang(
            base_vals(),
            &["x", "y", "z"],
            &[
                &["ineg", "iabs", "uextend", "sextend", "ireduce"],
                &["udiv", "sdiv", "urem", "srem"],
            ],
        ),
        all_rules.clone(),
    );
    all_rules.extend(div_rem);
    info!("div/rem rules are synthesized");

    let full_atoms = atoms.saturating_sub(1).max(1);
    let full = clif_rules(
        Metric::Atoms,
        full_atoms,
        lang(
            base_vals(),
            &["x", "y", "z"],
            &[
                &[
                    "ineg", "iabs", "bnot", "ctz", "clz", "cls", "popcnt", "uextend", "sextend",
                    "ireduce",
                ],
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

    let nested_bops_arith = Workload::new(&["(bop ty e e)", "v"])
        .plug("e", &Workload::new(&["(bop ty v v)", "(uop ty v)", "v"]))
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

    let nested_bops_full = Workload::new(&["(bop ty e e)", "v"])
        .plug("e", &Workload::new(&["(bop ty v v)", "(uop ty v)", "v"]))
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
        "(select ty V V V)",
        "(select ty V (OP ty V V) V)",
        "(select ty V V (OP ty V V))",
        "(select ty V (OP ty V V) (OP ty V V))",
        "(OP ty V (select ty V V V))",
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
