use crate::ast::Expr;
use chumsky::prelude::*;

#[allow(clippy::let_and_return)]
pub fn parser<'src>() -> impl Parser<'src, &'src str, Expr, extra::Err<Simple<'src, char>>> {
    let ident = text::ascii::ident().padded();

    let expr = recursive(|expr| {
        let lambda = just('\\')
            .ignore_then(ident.clone())
            .then_ignore(just("."))
            .then(expr.clone())
            .map(|(param, body): (&str, Expr)| Expr::lambda(param, body))
            .labelled("lambda");

        let apply = expr
            .clone()
            .then(expr.clone())
            .delimited_by(just('('), just(')'))
            .map(|(func, arg)| Expr::apply(func, arg));

        let let_binding = just("let")
            .ignore_then(ident.clone())
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just("in"))
            .then(expr.clone())
            .map(|((name, value), body): ((&str, Expr), Expr)| {
                Expr::apply(Expr::lambda(name, body), value)
            });

        let var = ident.clone().map(|name: &str| Expr::var(name));

        choice((lambda, apply, let_binding, var)).padded()
    });

    expr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda() {
        let expr = r"\x. x";
        let parsed = parser().parse(expr).into_result();
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), r"\x . x");
    }

    #[test]
    fn test_multiple_lines() {
        let expr = r"
            \x
            . x
        ";
        let parsed = parser().parse(expr).into_result();
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), r"\x . x");
    }

    #[test]
    fn test_application() {
        let expr = r"(\x. x y)";
        let parsed = parser().parse(expr).into_result();
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), r"(\x . x y)");
    }

    #[test]
    fn test_let_binding() {
        let expr = r"
            let id = \x. x in
            \id . id
        ";
        let parsed = parser().parse(expr).into_result();
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), r"(\id . \id . id \x . x)");
    }
}
