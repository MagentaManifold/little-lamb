use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Lambda {
    pub param: String,
    pub body: Box<Expr>,
}

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\\{} . {}", self.param, self.body)
    }
}

#[derive(Debug, Clone)]
pub struct Apply {
    pub func: Box<Expr>,
    pub arg: Box<Expr>,
}

impl Display for Apply {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} {})", self.func, self.arg)
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(String),
    Lambda(Lambda),
    Apply(Apply),
}

impl Expr {
    /// Create a new variable expression
    pub fn var(name: impl Into<String>) -> Self {
        Expr::Var(name.into())
    }

    /// Create a new lambda expression
    pub fn lambda(param: impl Into<String>, body: Expr) -> Self {
        Expr::Lambda(Lambda {
            param: param.into(),
            body: Box::new(body),
        })
    }

    /// Create a new application expression
    pub fn apply(func: Expr, arg: Expr) -> Self {
        Expr::Apply(Apply {
            func: Box::new(func),
            arg: Box::new(arg),
        })
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Var(name) => write!(f, "{name}"),
            Expr::Lambda(lambda) => write!(f, "{lambda}"),
            Expr::Apply(apply) => write!(f, "{apply}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Term {
    Var { name: String, index: usize },
    Apply { func: Box<Term>, arg: Box<Term> },
    Lambda { param: String, body: Box<Term> },
}

impl Term {
    pub fn var(name: impl Into<String>, index: usize) -> Self {
        Term::Var {
            name: name.into(),
            index,
        }
    }

    pub fn lambda(param: impl Into<String>, body: Term) -> Self {
        Term::Lambda {
            param: param.into(),
            body: Box::new(body),
        }
    }

    pub fn apply(func: Term, arg: Term) -> Self {
        Term::Apply {
            func: Box::new(func),
            arg: Box::new(arg),
        }
    }
}

impl Into<Expr> for Term {
    fn into(self) -> Expr {
        match self {
            Term::Var { name, .. } => Expr::var(name),
            Term::Lambda { param, body } => Expr::lambda(param, (*body).into()),
            Term::Apply { func, arg } => Expr::apply((*func).into(), (*arg).into()),
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Term::Var { name, index } => write!(f, "{name}#{index}"),
            Term::Lambda { param, body } => write!(f, "\\{param}. {body}"),
            Term::Apply { func, arg } => write!(f, "({func} {arg})"),
        }
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Term::Var { name: _, index: i1 }, Term::Var { name: _, index: i2 }) => i1 == i2,
            (Term::Lambda { param: _, body: b1 }, Term::Lambda { param: _, body: b2 }) => b1 == b2,
            (Term::Apply { func: f1, arg: a1 }, Term::Apply { func: f2, arg: a2 }) => {
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
    fn test_equal() {
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
