use thiserror::Error;

#[derive(Error, Debug)]
pub enum BoolExprParseError {
    #[error("found invalid brackets, only parentheses are supported in boolean expressions")]
    InvalidBrackets,
    #[error("found non-alphanumeric token in boolean expression: '{token}'")]
    NonAlphanumericToken { token: String },
    #[error("found consecutive terminals in boolean expression (expected operator between them), second was: '{second:?}'")]
    ConsecutiveTerminals {
        second: boolean_expression::Expr<String>,
    },
    #[error("found consecutive operators in boolean expression (expected terminal between them), second was: '{second}'")]
    ConsecutiveOperators { second: String },
    #[error("too many nested bracketed expressions found in boolean expression, please simplify your expression (this is a security measure)")]
    TooMuchNesting,
    #[error("found trailing operator '{op}' at end of boolean expression, expected terminal")]
    TrailingOperator { op: String },
    #[error("found empty stack in boolean expression (either empty parentheses or a completely empty expression)")]
    EmptyStack,
    #[error("found unmatched bracket in boolean expression: '{expr}'")]
    UnmatchedBracket { expr: String },
    #[error("found terminal '{terminal}', which was not in list of allowed terminals in boolean expression")]
    UnknownTerminal { terminal: String },
}
