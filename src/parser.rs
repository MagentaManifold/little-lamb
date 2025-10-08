use crate::ast::Expr;
use crate::lexer::{Token, lexer};
use chumsky::prelude::*;

#[allow(clippy::let_and_return)]
fn token_parser<'tokens>()
-> impl Parser<'tokens, &'tokens [Token], Expr, extra::Err<Rich<'tokens, Token>>> {
    let ident = select! {
        Token::Ident(name) => name,
    }
    .labelled("identifier");

    let expr = recursive(|expr| {
        let var = ident.clone().map(Expr::var);

        let paren = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .labelled("parenthesized expression");

        let atom = choice((paren.clone(), var));

        let apply = atom
            .clone()
            .foldl(atom.clone().repeated(), |acc, arg| Expr::apply(acc, arg))
            .labelled("application");

        let lambda = just(Token::Lambda)
            .ignore_then(ident.clone().repeated().at_least(1).collect::<Vec<_>>())
            .then_ignore(just(Token::Dot))
            .then(expr.clone())
            .map(|(params, body)| {
                params
                    .iter()
                    .rev()
                    .fold(body, |acc, param| Expr::lambda(param, acc))
            })
            .labelled("lambda");

        let assign = ident
            .clone()
            .then_ignore(just(Token::Equal))
            .then(expr.clone());

        let let_binding = just(Token::Let)
            .ignore_then(
                assign
                    .separated_by(just(Token::Comma))
                    .allow_leading()
                    .allow_trailing()
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .map(|(assignments, body)| {
                assignments
                    .into_iter()
                    .rev()
                    .fold(body, |acc, (name, value)| {
                        Expr::apply(Expr::lambda(name, acc), value)
                    })
            })
            .labelled("let binding");
        choice((apply, lambda, let_binding, atom))
    });

    expr
}

pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = lexer()
        .parse(input)
        .into_result()
        .map_err(|errs| format!("Lexer errors: {:?}", errs))?;
    let expr = token_parser()
        .parse(&tokens)
        .into_result()
        .map_err(|errs| format!("Parser errors: {:?}", errs))?;
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda() {
        let expr = r"\x. x";
        let parsed = parse(expr);
        let expected = Expr::lambda("x", Expr::var("x"));
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_lambda_currying() {
        let expr = r"\x y. x";
        let parsed = parse(expr);
        let expected = Expr::lambda("x", Expr::lambda("y", Expr::var("x")));
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_multiple_lines() {
        let expr = r"
            \x
            . x
        ";
        let parsed = parse(expr);
        let expected = Expr::lambda("x", Expr::var("x"));
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_application() {
        let expr = r"\x y. x y";
        let parsed = parse(expr);
        let expected = Expr::lambda(
            "x",
            Expr::lambda("y", Expr::apply(Expr::var("x"), Expr::var("y"))),
        );
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_deep_application() {
        let expr = r"\x y z. x y z";
        let parsed = parse(expr);
        let expected = Expr::lambda(
            "x",
            Expr::lambda(
                "y",
                Expr::lambda(
                    "z",
                    Expr::apply(Expr::apply(Expr::var("x"), Expr::var("y")), Expr::var("z")),
                ),
            ),
        );
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_let_binding() {
        let expr = r"
            let id = \x. x in
            \id. id
        ";
        let parsed = parse(expr);
        let expected = Expr::apply(
            Expr::lambda("id", Expr::lambda("id", Expr::var("id"))),
            Expr::lambda("x", Expr::var("x")),
        );
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_let_binding_with_comma_trailing() {
        let expr_comma = r"
            let
                I = \x. x,
                K = \x y. x,
            in
            K I
        ";
        let expr_nested = r"
            let I = \x. x in
            let K = \x y. x in
            K I
        ";
        let parsed_comma = parse(expr_comma);
        let parsed_nested = parse(expr_nested);
        assert_eq!(parsed_comma.unwrap(), parsed_nested.unwrap());
    }

    #[test]
    fn test_let_binding_with_comma_leading() {
        let expr_comma = r"
            let
            , I = \x. x
            , K = \x y. x
            in
            K I
        ";
        let expr_nested = r"
            let I = \x. x in
            let K = \x y. x in
            K I
        ";
        let parsed_comma = parse(expr_comma);
        let parsed_nested = parse(expr_nested);
        assert_eq!(parsed_comma.unwrap(), parsed_nested.unwrap());
    }

    #[test]
    fn test_params() {
        let expr = r"((\x. (x)))";
        let parsed = parse(expr);
        let expected = Expr::lambda("x", Expr::var("x"));
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_comments() {
        let expr = r"
            -- this is a comment
            let id = \x. x in -- another comment
            \id. id --end comment
        ";
        let parsed = parse(expr);
        let expected = Expr::apply(
            Expr::lambda("id", Expr::lambda("id", Expr::var("id"))),
            Expr::lambda("x", Expr::var("x")),
        );
        assert_eq!(parsed.unwrap(), expected);
    }
}
