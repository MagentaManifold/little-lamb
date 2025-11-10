use std::fmt::Display;
use std::rc::Rc;

use super::Expr;

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
        enum Work {
            ConvertTerm(Term),
            BuildLambda(String),
            BuildApply,
        }

        let mut work_stack: Vec<Work> = vec![Work::ConvertTerm(term)];
        let mut result_stack: Vec<Expr> = Vec::new();

        while let Some(work) = work_stack.pop() {
            match work {
                Work::ConvertTerm(term) => match term.inner() {
                    TermInner::Var { name, .. } => {
                        result_stack.push(Expr::var(name.to_string()));
                    }
                    TermInner::Lambda { param, body } => {
                        work_stack.push(Work::BuildLambda(param.to_string()));
                        work_stack.push(Work::ConvertTerm(body.clone()));
                    }
                    TermInner::Apply { func, arg } => {
                        work_stack.push(Work::BuildApply);
                        work_stack.push(Work::ConvertTerm(arg.clone()));
                        work_stack.push(Work::ConvertTerm(func.clone()));
                    }
                },
                Work::BuildLambda(param) => {
                    let body = result_stack.pop().expect("Missing body for lambda");
                    result_stack.push(Expr::lambda(param, body));
                }
                Work::BuildApply => {
                    let arg = result_stack.pop().expect("Missing arg for apply");
                    let func = result_stack.pop().expect("Missing func for apply");
                    result_stack.push(Expr::apply(func, arg));
                }
            }
        }

        result_stack
            .pop()
            .expect("Conversion should produce exactly one result")
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
    use super::Term as T;

    #[test]
    fn test_term_equal() {
        let t1 = T::var("x", 0);
        let t2 = T::var("y", 0);
        let t3 = T::var("x", 1);
        assert_eq!(t1, t2);
        assert_ne!(t1, t3);

        let l1 = T::lambda("x", T::var("x", 0));
        let l2 = T::lambda("y", T::var("y", 0));
        let l3 = T::lambda("x", T::var("x", 1));
        assert_eq!(l1, l2);
        assert_ne!(l1, l3);

        let a1 = T::apply(T::lambda("x", T::var("x", 0)), T::var("y", 0));
        let a2 = T::apply(T::lambda("z", T::var("z", 0)), T::var("w", 0));
        let a3 = T::apply(T::lambda("x", T::var("x", 1)), T::var("y", 0));
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }
}
