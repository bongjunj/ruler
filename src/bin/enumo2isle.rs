use std::{fs, path::PathBuf};

use clap::Parser;
use ruler::enumo::Sexp;

#[derive(Parser)]
struct Args {
    input: PathBuf,
}

fn main() {
    let args = Args::parse();

    let input = fs::read_to_string(&args.input).unwrap();

    for line in input.lines() {
        let rule = convert_line(line);
        println!("{rule}");
    }
}

fn convert_line(line: &str) -> String {
    let (lhs, rhs) = split_rule(line);
    let lhs = parse_sexp(lhs.trim());
    let rhs = parse_sexp(rhs.trim());
    // TODO: for urem/srem/udiv/sdiv -> simplify_skeleton
    format!(
        "(rule (simplify {}) {})",
        format_isle(&lhs),
        format_isle(&rhs)
    )
}

fn split_rule(line: &str) -> (&str, &str) {
    line.split_once("<=>")
        .or_else(|| line.split_once("==>"))
        .unwrap()
}

fn parse_sexp(input: &str) -> Sexp {
    symbolic_expressions::parser::parse_str(input)
        .map(from_symbolic_expr)
        .unwrap()
}

fn from_symbolic_expr(sexp: symbolic_expressions::Sexp) -> Sexp {
    match sexp {
        symbolic_expressions::Sexp::String(s) => Sexp::Atom(s),
        symbolic_expressions::Sexp::List(items) => {
            Sexp::List(items.into_iter().map(from_symbolic_expr).collect())
        }
        symbolic_expressions::Sexp::Empty => Sexp::List(vec![]),
    }
}

fn format_isle(sexp: &Sexp) -> String {
    match sexp {
        Sexp::Atom(atom) => format_atom(atom),
        Sexp::List(items) => {
            let (head, args) = items.split_first().unwrap();
            let mut parts = Vec::with_capacity(items.len() + 1);
            parts.push(head.to_string());
            parts.extend(args.iter().map(format_isle));
            return format!("({})", parts.join(" "));
        }
    }
}

// a variable (?a, ?b), type, a literal
fn format_atom(atom: &str) -> String {
    if let Some(var) = atom.strip_prefix('?') {
        return var.to_string();
    }

    if let Some(n) = atom.parse::<i64>().ok() {
        if n < 0 {
            return format!("(iconst_s ty {n})");
        }
        return format!("(iconst_u ty {n})");
    }

    if matches!(atom, "ty" | "ty1" | "ty2") {
        return atom.to_string();
    }

    unreachable!("an atom is a variable or a literal: {}", atom)
}

#[cfg(test)]
mod enumo2isle_tests {
    use crate::convert_line;

    #[test]
    fn add_zero_cancels() {
        let result = convert_line(&"(iadd ty ?a 0) <=> ?a");
        assert_eq!(result, "(rule (simplify (iadd ty a (iconst_u ty 0))) a)")
    }

    #[test]
    fn eq_zero_to_ult_1() {
        let result = convert_line(&"(eq ty ?a 0) <=> (ult ty ?a 1)");
        assert_eq!(
            result,
            "(rule (simplify (eq ty a (iconst_u ty 0))) (ult ty a (iconst_u ty 1)))"
        )
    }

    #[test]
    fn sub_zero_cancels() {
        let result = convert_line(&"(isub ty ?a 0) <=> ?a");
        assert_eq!(result, "(rule (simplify (isub ty a (iconst_u ty 0))) a)")
    }
}
