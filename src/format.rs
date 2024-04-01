use boolean_expression::Expr;

/// Converts the given boolean expression into a string through recursion.
pub fn format_bool_expr(expr: Expr<String>) -> String {
    let expr = expr.simplify_via_laws();

    // Inner recursion function to avoid bracket stripping so we can maintain unambiguous
    // order of operations
    fn format(expr: Expr<String>) -> String {
        match expr {
            Expr::And(lhs, rhs) => format!("({} & {})", format(*lhs), format(*rhs)),
            Expr::Or(lhs, rhs) => format!("({} | {})", format(*lhs), format(*rhs)),
            Expr::Not(inner) => format!("!{}", format(*inner)),
            Expr::Terminal(s) => s,
            Expr::Const(b) => if b { "true" } else { "false" }.to_string(),
        }
    }

    let formatted = format(expr);
    // Remove leading and trailing brackets if necessary
    // NOTE: Something like `(x | y) & z` would be `((x | y) & z)` by here
    if formatted.starts_with('(') {
        formatted[1..formatted.len() - 1].to_string()
    } else {
        formatted
    }
}
