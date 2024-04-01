use super::error::BoolExprParseError;
use boolean_expression::Expr;

/// Parses the given string into a boolean expression.
///
/// This uses a maximum nesting of 100, which can be set to a custom value with [`parse_bool_expr_str_with_max_nesting`].
#[inline]
pub fn parse_bool_expr_str(
    raw_expr_str: &str,
    allowed_terminals: &[&str],
) -> Result<Expr<String>, BoolExprParseError> {
    parse_bool_expr_str_with_max_nesting(raw_expr_str, allowed_terminals, 100)
}

/// Parses the given string into a boolean expression.
///
/// A maximum degree of nesting must be set in order to prevent an attacker from causing excessive
/// memory use through infinite bracketing. Note that this would *not* trigger a stack overflow,
/// it would trigger an out-of-memory error, eventually, after significant stagnation. A default value
/// can be used for simplicity (and brevity) with `parse_bool_expr_str`
pub fn parse_bool_expr_str_with_max_nesting(
    raw_expr_str: &str,
    allowed_terminals: &[&str],
    max_nesting: usize,
) -> Result<Expr<String>, BoolExprParseError> {
    // Replace logic operators to make everything uniform: `&` for and, `|` for or, and `!`
    // for not. Also add space around brackets so they can be treated as independent tokens.
    // Weird spacing here to avoid messing up terminal names.
    let expr_str = raw_expr_str
        .to_lowercase()
        .replace(" and", " &")
        .replace(" &&", " &")
        .replace(" or", " |")
        .replace(" ||", " |")
        .replace(" not ", " !")
        .replace("(", " ( ")
        .replace(")", " ) ");
    // We haven't handled `not` at the start of the expression
    let expr_str = if expr_str.starts_with("not ") {
        format!("!{}", &expr_str[3..])
    } else {
        expr_str
    };

    // Make sure we don't have illegal brackets
    if expr_str.contains('[')
        || expr_str.contains(']')
        || expr_str.contains('{')
        || expr_str.contains('}')
        || expr_str.contains('<')
        || expr_str.contains('>')
    {
        return Err(BoolExprParseError::InvalidBrackets);
    }
    // Split things up into tokens (removing any double whitespace)
    let tokens: Vec<&str> = expr_str.split(' ').map(|tok| tok.trim()).collect();

    // Boolean expressions are built on unary negation operators and binary operations, together
    // with brackets that start new sub-expressions. Hence, we accumulate non-bracketed expressions
    // into the newest stack, and collapse them as brackets are closed
    let mut stacks = vec![TokenStack::default()];
    for tok in tokens {
        // Make sure we haven't got too much nesting
        if stacks.len() > max_nesting {
            return Err(BoolExprParseError::TooMuchNesting);
        }
        if tok.is_empty() {
            continue;
        }

        match tok {
            "(" => stacks.push(TokenStack::default()),
            ")" => {
                if stacks.len() > 1 {
                    let unwound = stacks.remove(stacks.len() - 1);
                    let new_last = stacks.last_mut().unwrap();
                    new_last.push(unwound.finish()?)?;
                } else {
                    return Err(BoolExprParseError::UnmatchedBracket {
                        expr: raw_expr_str.to_string(),
                    });
                }
            }
            "&" => stacks.last_mut().unwrap().push_op(BoolOp::And)?,
            "|" => stacks.last_mut().unwrap().push_op(BoolOp::Or)?,
            // We have to have support for constants, because they might be written back after logical
            // simplification
            "true" => stacks.last_mut().unwrap().push(Expr::Const(true))?,
            "false" => stacks.last_mut().unwrap().push(Expr::Const(false))?,
            tok => stacks
                .last_mut()
                .unwrap()
                .push_str(tok, allowed_terminals)?,
        }
    }

    // All sub-stacks should have been collapsed into the primary stack
    if stacks.len() > 1 {
        return Err(BoolExprParseError::UnmatchedBracket {
            expr: raw_expr_str.to_string(),
        });
    }
    // This will work provided there isn't a trailing operator
    let expr = stacks.remove(0).finish()?;

    // We'll automatically try to perform elementary simplification (cheap operation)
    Ok(expr.simplify_via_laws())
}

/// Intermediate parsing infrastructure for tokens in boolean expressions.
#[derive(Default)]
struct TokenStack {
    expr: Option<Expr<String>>,
    op: Option<BoolOp>,
}
impl TokenStack {
    /// Pushes the given new token string onto the stack. This is simply a wrapper for token parsing and
    /// calling [`Self::push`].
    fn push_str(
        &mut self,
        token: &str,
        allowed_terminals: &[&str],
    ) -> Result<(), BoolExprParseError> {
        // Strip not modifiers
        let (token, is_negated) = if token.starts_with('!') {
            (&token[1..], true)
        } else {
            (token, false)
        };
        // Make sure the token is valid to avoid issues with stray modifiers
        if token.chars().any(|c| !c.is_alphanumeric() && c != '_') {
            return Err(BoolExprParseError::NonAlphanumericToken {
                token: token.to_string(),
            });
        }

        // Make sure this is a legal terminal
        if !allowed_terminals.contains(&token) {
            return Err(BoolExprParseError::UnknownTerminal {
                terminal: token.to_string(),
            });
        }

        // It is certain that `self.right` is `None`
        let right_expr = if is_negated {
            Expr::Not(Box::new(Expr::Terminal(token.to_string())))
        } else {
            Expr::Terminal(token.to_string())
        };

        self.push(right_expr)
    }
    /// Pushes the given expression onto the stack. This will fail if the stack has not had an operator
    /// pushed onto it (provided the stack is non-empty, otherwise this will just become the first
    /// element).
    fn push(&mut self, right: Expr<String>) -> Result<(), BoolExprParseError> {
        if self.expr.is_none() {
            self.expr = Some(right);

            Ok(())
        } else if self.op.is_none() {
            Err(BoolExprParseError::ConsecutiveTerminals { second: right })
        } else {
            // We have a left expression and an operator; we definitely don't have a right expression,
            // because we automatically combine it in this function
            self.expr = Some(match self.op {
                // If there is no left expression, `true & x = x`
                Some(BoolOp::And) => {
                    std::mem::take(&mut self.expr).unwrap_or(Expr::Const(true)) & right
                }
                // If there is no left expression `false | y = y`
                Some(BoolOp::Or) => {
                    std::mem::take(&mut self.expr).unwrap_or(Expr::Const(false)) | right
                }
                None => return Err(BoolExprParseError::ConsecutiveTerminals { second: right }),
            });
            self.op = None;

            Ok(())
        }
    }
    /// Pushes the given operator onto the stack, provided the stack is non-empty.
    fn push_op(&mut self, op: BoolOp) -> Result<(), BoolExprParseError> {
        if self.op.is_some() {
            // After combination, this could still happen if there was no right expression, meaning
            // we have consecutive operators
            Err(BoolExprParseError::ConsecutiveOperators { second: op.into() })
        } else {
            // We know `self.right` is `None` by how operators are pushed
            self.op = Some(op);
            Ok(())
        }
    }
    /// Finalises the token stack and converts it into a final expression. This will fail if there
    /// is an operator without a right expression to combine it with. This will also fail if the stack
    /// is empty
    fn finish(self) -> Result<Expr<String>, BoolExprParseError> {
        if let Some(op) = self.op {
            Err(BoolExprParseError::TrailingOperator { op: op.into() })
        } else if let Some(expr) = self.expr {
            Ok(expr)
        } else {
            Err(BoolExprParseError::EmptyStack)
        }
    }
}

enum BoolOp {
    And,
    Or,
}
impl Into<String> for BoolOp {
    fn into(self) -> String {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
        .to_string()
    }
}
