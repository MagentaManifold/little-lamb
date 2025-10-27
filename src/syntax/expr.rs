use std::fmt::Display;

use super::Term;
use crate::eval::{EvalError, de_bruijn};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Lambda { param: String, body: Box<Expr> },
    Apply { func: Box<Expr>, arg: Box<Expr> },
}

impl Expr {
    /// Create a new variable expression
    pub fn var(name: impl Into<String>) -> Self {
        Expr::Var(name.into())
    }

    /// Create a new lambda expression
    pub fn lambda(param: impl Into<String>, body: Expr) -> Self {
        Expr::Lambda {
            param: param.into(),
            body: Box::new(body),
        }
    }

    /// Create a new application expression
    pub fn apply(func: Expr, arg: Expr) -> Self {
        Expr::Apply {
            func: Box::new(func),
            arg: Box::new(arg),
        }
    }
}

impl Expr {
    /// Try to convert this expression to a Church numeral natural number
    pub fn to_nat(&self) -> Option<usize> {
        let Expr::Lambda {
            param: f_param,
            body: f_body,
        } = self
        else {
            return None;
        };

        let Expr::Lambda {
            param: x_param,
            body: x_body,
        } = &**f_body
        else {
            return None;
        };

        let mut count = 0;
        let mut current = x_body.as_ref();

        loop {
            match current {
                Expr::Var(name) if name == x_param => {
                    return Some(count);
                }
                Expr::Apply { func, arg } => {
                    let Expr::Var(func_name) = &**func else {
                        return None;
                    };

                    if func_name != f_param {
                        return None;
                    }

                    count += 1;
                    current = arg;
                }
                _ => return None,
            }
        }
    }

    /// Try to recognize this expression as a common function by comparing
    /// against the library files
    pub fn to_lib_function(&self) -> Option<&'static str> {
        use crate::import::get_lib_function_names;

        let Ok(self_term) = Term::try_from(self.clone()) else {
            return None;
        };

        for name in get_lib_function_names() {
            if let Ok(lib_term) = crate::import::load_lib_function_as_term(name) {
                if self_term == lib_term {
                    return Some(name);
                }
            }
        }

        None
    }

    /// Display the expression in a readable format, converting to natural numbers
    /// and library functions when possible
    pub fn try_into_readable_string(&self) -> Option<String> {
        if let Some(n) = self.to_nat() {
            return Some(n.to_string());
        }

        if let Some(lib_fn) = self.to_lib_function() {
            return Some(lib_fn.to_string());
        }

        None
    }

    /// Display the expression in a readable format, converting to natural numbers
    /// and library functions when possible. Falls back to standard notation.
    pub fn to_readable_string(&self) -> String {
        if let Some(readable) = self.try_into_readable_string() {
            return readable;
        }

        self.to_string()
    }

    pub fn to_string_with_depth(&self, depth: usize) -> String {
        match self {
            Expr::Var(name) => name.clone(),
            Expr::Lambda { param, body } => {
                if depth == 0 {
                    "...".to_string()
                } else {
                    format!("\\{} . {}", param, body.to_string_with_depth(depth - 1))
                }
            }
            Expr::Apply { func, arg } => {
                if depth == 0 {
                    "...".to_string()
                } else {
                    let func_str = match **func {
                        Expr::Var(_) | Expr::Apply { .. } => func.to_string_with_depth(depth - 1),
                        _ => format!("({})", func.to_string_with_depth(depth)),
                    };
                    let arg_str = match **arg {
                        Expr::Var(_) => arg.to_string_with_depth(depth - 1),
                        _ => format!("({})", arg.to_string_with_depth(depth - 1)),
                    };
                    format!("{} {}", func_str, arg_str)
                }
            }
        }
    }

    pub fn to_readable_string_with_depth(&self, depth: usize) -> String {
        if let Some(readable) = self.try_into_readable_string() {
            return readable;
        }
        match self {
            Expr::Var(name) => name.clone(),
            Expr::Lambda { param, body } => {
                if depth == 0 {
                    "...".to_string()
                } else {
                    format!(
                        "\\{} . {}",
                        param,
                        body.to_readable_string_with_depth(depth - 1)
                    )
                }
            }
            Expr::Apply { func, arg } => {
                if depth == 0 {
                    "...".to_string()
                } else {
                    let func_str = if let Some(readable_func) = func.try_into_readable_string() {
                        readable_func
                    } else {
                        match **func {
                            Expr::Var(_) | Expr::Apply { .. } => {
                                func.to_readable_string_with_depth(depth - 1)
                            }
                            _ => format!("({})", func.to_readable_string_with_depth(depth - 1)),
                        }
                    };
                    let arg_str = if let Some(readable_arg) = arg.try_into_readable_string() {
                        readable_arg
                    } else {
                        match **arg {
                            Expr::Var(_) => arg.to_readable_string_with_depth(depth - 1),
                            _ => format!("({})", arg.to_readable_string_with_depth(depth - 1)),
                        }
                    };
                    format!("{} {}", func_str, arg_str)
                }
            }
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Var(name) => write!(f, "{name}"),
            Expr::Lambda { param, body } => write!(f, "\\{param} . {body}"),
            Expr::Apply { func, arg } => {
                let func_str = match **func {
                    Expr::Var(_) | Expr::Apply { .. } => func.to_string(),
                    _ => format!("({func})"),
                };
                let arg_str = match **arg {
                    Expr::Var(_) => arg.to_string(),
                    _ => format!("({arg})"),
                };
                write!(f, "{func_str} {arg_str}")
            }
        }
    }
}

impl TryFrom<Expr> for Term {
    type Error = EvalError;
    fn try_from(expr: Expr) -> Result<Self, Self::Error> {
        de_bruijn(&expr, &mut Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::Importer;
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::syntax::desugar;
    use std::path::Path;

    /// Helper function to parse a string into an Expr
    fn parse_expr(src: &str) -> Expr {
        let tokens = tokenize(src).unwrap();
        let ast = parse(&tokens).unwrap();
        desugar(ast, &mut Importer::new(), Some(Path::new("."))).unwrap()
    }

    #[test]
    fn test_expr_to_nat_zero() {
        let zero = parse_expr(r"\f x. x");
        assert_eq!(zero.to_nat(), Some(0));
        assert_eq!(zero.to_readable_string(), "0");
    }

    #[test]
    fn test_expr_to_nat_three() {
        let three = parse_expr(r"\f x. f (f (f x))");
        assert_eq!(three.to_nat(), Some(3));
        assert_eq!(three.to_readable_string(), "3");
    }

    #[test]
    fn test_expr_to_lib_function_k_or_true() {
        let k_expr = parse_expr(r"\x y. x");
        assert_eq!(k_expr.to_lib_function(), Some("K"));
        assert_eq!(k_expr.to_readable_string(), "K");
    }

    #[test]
    fn test_expr_to_nat_zero_or_false() {
        // \x y. y is both Church numeral 0 and false
        // We prioritize the natural number interpretation
        let zero_or_false = parse_expr(r"\x y. y");
        assert_eq!(zero_or_false.to_nat(), Some(0));
        assert_eq!(zero_or_false.to_readable_string(), "0");
    }

    #[test]
    fn test_expr_regualar_display() {
        let not_numeral = parse_expr(r"\x y. y x");
        assert_eq!(not_numeral.to_nat(), None);
        assert_eq!(not_numeral.to_readable_string(), "\\x . \\y . y x");
    }
}
