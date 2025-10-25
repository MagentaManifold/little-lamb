use std::fmt::Display;
use std::rc::Rc;

use crate::eval::{EvalError, de_bruijn};

#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    Var(String),
    Nat(usize),
    Lambda {
        param: String,
        body: Box<Ast>,
    },
    Apply {
        func: Box<Ast>,
        arg: Box<Ast>,
    },
    Let {
        name: String,
        value: Box<Ast>,
        body: Box<Ast>,
    },
    Import {
        module: String,
        name: String,
        body: Box<Ast>,
    },
}

impl Ast {
    /// Create a new variable AST node
    pub fn var(name: impl Into<String>) -> Self {
        Ast::Var(name.into())
    }

    /// Create a new natural number AST node
    pub fn nat(value: usize) -> Self {
        Ast::Nat(value)
    }

    /// Create a new lambda AST node
    pub fn lambda(param: impl Into<String>, body: Ast) -> Self {
        Ast::Lambda {
            param: param.into(),
            body: Box::new(body),
        }
    }

    /// Create a new application AST node
    pub fn apply(func: Ast, arg: Ast) -> Self {
        Ast::Apply {
            func: Box::new(func),
            arg: Box::new(arg),
        }
    }

    /// Create a new let binding AST node
    pub fn let_binding(name: impl Into<String>, value: Ast, body: Ast) -> Self {
        Ast::Let {
            name: name.into(),
            value: Box::new(value),
            body: Box::new(body),
        }
    }

    /// Create a new import AST node
    pub fn import(module: impl Into<String>, name: impl Into<String>, body: Ast) -> Self {
        Ast::Import {
            module: module.into(),
            name: name.into(),
            body: Box::new(body),
        }
    }
}

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

/// Term's internal enum that's always wrapped in Rc
#[derive(Debug)]
pub enum TermInner {
    Var { name: Rc<str>, index: usize },
    Apply { func: Term, arg: Term },
    Lambda { param: Rc<str>, body: Term },
}

/// The Term struct represents terms in de Bruijn indexed form, wrapped in Rc
/// for efficient cloning.
#[derive(Debug, Clone)]
pub struct Term(Rc<TermInner>);

impl Term {
    pub fn var(name: impl Into<Rc<str>>, index: usize) -> Self {
        Term(Rc::new(TermInner::Var {
            name: name.into(),
            index,
        }))
    }

    pub fn lambda(param: impl Into<Rc<str>>, body: Term) -> Self {
        Term(Rc::new(TermInner::Lambda {
            param: param.into(),
            body,
        }))
    }

    pub fn apply(func: Term, arg: Term) -> Self {
        Term(Rc::new(TermInner::Apply { func, arg }))
    }

    // Helper method to access the inner value
    pub fn inner(&self) -> &TermInner {
        &self.0
    }
}

impl From<Term> for Expr {
    fn from(term: Term) -> Self {
        match term.inner() {
            TermInner::Var { name, .. } => Expr::var(name.to_string()),
            TermInner::Lambda { param, body } => {
                Expr::lambda(param.to_string(), body.clone().into())
            }
            TermInner::Apply { func, arg } => Expr::apply(func.clone().into(), arg.clone().into()),
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.inner() {
            TermInner::Var { name, index } => write!(f, "{name}#{index}"),
            TermInner::Lambda { param, body } => write!(f, "\\{param}. {body}"),
            TermInner::Apply { func, arg } => {
                let func_str = match func.inner() {
                    TermInner::Var { .. } | TermInner::Apply { .. } => func.to_string(),
                    _ => format!("({func})"),
                };
                let arg_str = match arg.inner() {
                    TermInner::Var { .. } => arg.to_string(),
                    _ => format!("({arg})"),
                };
                write!(f, "{func_str} {arg_str}")
            }
        }
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        if Rc::ptr_eq(&self.0, &other.0) {
            return true;
        }

        match (self.inner(), other.inner()) {
            (TermInner::Var { name: _, index: i1 }, TermInner::Var { name: _, index: i2 }) => {
                i1 == i2
            }
            (
                TermInner::Lambda { param: _, body: b1 },
                TermInner::Lambda { param: _, body: b2 },
            ) => b1 == b2,
            (TermInner::Apply { func: f1, arg: a1 }, TermInner::Apply { func: f2, arg: a2 }) => {
                f1 == f2 && a1 == a2
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_equal() {
        let t1 = Term::var("x", 0);
        let t2 = Term::var("y", 0);
        let t3 = Term::var("x", 1);
        assert_eq!(t1, t2);
        assert_ne!(t1, t3);

        let l1 = Term::lambda("x", Term::var("x", 0));
        let l2 = Term::lambda("y", Term::var("y", 0));
        let l3 = Term::lambda("x", Term::var("x", 1));
        assert_eq!(l1, l2);
        assert_ne!(l1, l3);

        let a1 = Term::apply(Term::lambda("x", Term::var("x", 0)), Term::var("y", 0));
        let a2 = Term::apply(Term::lambda("z", Term::var("z", 0)), Term::var("w", 0));
        let a3 = Term::apply(Term::lambda("x", Term::var("x", 1)), Term::var("y", 0));
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }
}
