use super::{format_bool_expr, parse_bool_expr_str};

#[test]
fn should_parse_simple_expr() {
    let raw = "x and y and not z";
    let parsed = parse_bool_expr_str(raw, &["x", "y", "z"]).unwrap();

    assert_eq!(format_bool_expr(parsed), "(x & y) & !z");
}

#[test]
fn should_parse_and_simplify_complex_expr() {
    // Mix of syntax
    let raw = "x and (a || (b && !c))";
    let parsed = parse_bool_expr_str(raw, &["x", "a", "b", "c"]).unwrap();

    // This is significantly easier to understand the implications of
    assert_eq!(format_bool_expr(parsed), "(x & a) | (x & (b & !c))");
}

#[test]
fn should_work_for_constants() {
    let raw = "x | !x";
    // Advanced simplification here (expensive, but always an option if we control the expression complexity)
    let parsed = parse_bool_expr_str(raw, &["x"]).unwrap().simplify_via_bdd();
    let reformatted = format_bool_expr(parsed);

    assert_eq!(reformatted, "true");

    // Constants are not terminals
    let parsed = parse_bool_expr_str(&reformatted, &[]).unwrap();
    assert_eq!(format_bool_expr(parsed), "true");
}

#[test]
fn should_fail_on_unknown_terminals() {
    let raw = "x | y";
    assert!(parse_bool_expr_str(raw, &["y"]).is_err());
}
