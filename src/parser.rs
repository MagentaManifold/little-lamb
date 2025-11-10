use crate::lexer::Token;
use crate::syntax::Ast;
use chumsky::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Parser errors: {0:?}")]
pub struct ParserError(Vec<String>);

impl ParserError {
    fn new(errors: Vec<Rich<'_, Token>>) -> Self {
        ParserError(errors.into_iter().map(|e| e.to_string()).collect())
    }
}

#[allow(clippy::let_and_return)]
fn parser<'tokens>() -> impl Parser<'tokens, &'tokens [Token], Ast, extra::Err<Rich<'tokens, Token>>>
{
    let ident = select! {
        Token::Ident(name) => name,
    }
    .labelled("identifier");

    let ast = recursive(|ast| {
        let var = ident.clone().map(Ast::var);

        let boolean = select! {
            Token::Boolean(value) => Ast::boolean(value),
        }
        .labelled("boolean");

        let nat = select! {
            Token::Natural(value) => Ast::nat(value),
        }
        .labelled("natural number");

        let paren = ast
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .labelled("parenthesized expression");

        let atom = choice((paren, var, boolean, nat));

        let apply = atom
            .clone()
            .foldl(atom.clone().repeated(), |acc, arg| Ast::apply(acc, arg))
            .labelled("application");

        let lambda = just(Token::Lambda)
            .ignore_then(ident.clone().repeated().at_least(1).collect::<Vec<_>>())
            .then_ignore(just(Token::Dot))
            .then(ast.clone())
            .map(|(params, body)| {
                params
                    .into_iter()
                    .rev()
                    .fold(body, |acc, param| Ast::lambda(param, acc))
            })
            .labelled("lambda");

        let assign = ident
            .clone()
            .then_ignore(just(Token::Equal))
            .then(ast.clone());

        let let_binding = just(Token::Let)
            .ignore_then(
                assign
                    .clone()
                    .separated_by(just(Token::Comma))
                    .allow_leading()
                    .allow_trailing()
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(Token::In))
            .then(ast.clone())
            .map(|(bindings, body)| {
                bindings.into_iter().rev().fold(body, |acc, (name, value)| {
                    Ast::let_binding(name, value, acc)
                })
            })
            .labelled("let binding");

        let letrec_binding = just(Token::Letrec)
            .ignore_then(
                assign
                    .separated_by(just(Token::Comma))
                    .allow_leading()
                    .allow_trailing()
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(Token::In))
            .then(ast.clone())
            .map(|(bindings, body)| {
                bindings.into_iter().rev().fold(body, |acc, (name, value)| {
                    Ast::letrec_binding(name, value, acc)
                })
            })
            .labelled("letrec binding");

        let import_binding = choice((
            ident
                .clone()
                .then_ignore(just(Token::As))
                .then(ident.clone()),
            ident.clone().map(|module| (module.clone(), module)),
        ));

        let import = just(Token::Import)
            .ignore_then(
                import_binding
                    .separated_by(just(Token::Comma))
                    .allow_leading()
                    .allow_trailing()
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(Token::In))
            .then(ast.clone())
            .map(|(bindings, body)| {
                bindings
                    .into_iter()
                    .rev()
                    .fold(body, |acc, (module, name)| Ast::import(module, name, acc))
            })
            .labelled("import");
        choice((apply, lambda, let_binding, letrec_binding, import, atom))
    });

    ast
}

pub fn parse(tokens: &[Token]) -> Result<Ast, ParserError> {
    parser()
        .parse(tokens)
        .into_result()
        .map_err(ParserError::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn tokenize_and_parse(src: &str) -> Ast {
        let tokens = tokenize(src).unwrap();
        parse(&tokens).unwrap()
    }

    #[test]
    fn test_lambda() {
        let expr = r"\x. x";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::lambda("x", Ast::var("x"));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_lambda_currying() {
        let expr = r"\x y. x";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::lambda("x", Ast::lambda("y", Ast::var("x")));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_multiple_lines() {
        let expr = r"
            \x
            . x
        ";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::lambda("x", Ast::var("x"));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_application() {
        let expr = r"\x y. x y";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::lambda(
            "x",
            Ast::lambda("y", Ast::apply(Ast::var("x"), Ast::var("y"))),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_deep_application() {
        let expr = r"\x y z. x y z";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::lambda(
            "x",
            Ast::lambda(
                "y",
                Ast::lambda(
                    "z",
                    Ast::apply(Ast::apply(Ast::var("x"), Ast::var("y")), Ast::var("z")),
                ),
            ),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_let_binding() {
        let expr = r"
            let id = \x. x in
            \id. id
        ";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::let_binding(
            "id",
            Ast::lambda("x", Ast::var("x")),
            Ast::lambda("id", Ast::var("id")),
        );
        assert_eq!(parsed, expected);
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
        let parsed_comma = tokenize_and_parse(expr_comma);
        let parsed_nested = tokenize_and_parse(expr_nested);
        assert_eq!(parsed_comma, parsed_nested);
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
        let parsed_comma = tokenize_and_parse(expr_comma);
        let parsed_nested = tokenize_and_parse(expr_nested);
        assert_eq!(parsed_comma, parsed_nested);
    }

    #[test]
    fn test_import() {
        let expr = r"
            import I in
            \x. I x
        ";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::import(
            "I",
            "I",
            Ast::lambda("x", Ast::apply(Ast::var("I"), Ast::var("x"))),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_import_as() {
        let expr = r"
            import I as id in
            \x. id x
        ";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::import(
            "I",
            "id",
            Ast::lambda("x", Ast::apply(Ast::var("id"), Ast::var("x"))),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_import_multiple() {
        let expr_comma = r"
            import I as id, K in
            \x y. id (K x y)
        ";
        let expr_nested = r"
            import I as id in
            import K in
            \x y. id (K x y)
        ";
        let parsed_comma = tokenize_and_parse(expr_comma);
        let parsed_nested = tokenize_and_parse(expr_nested);
        assert_eq!(parsed_comma, parsed_nested);
    }

    #[test]
    fn test_nat() {
        let expr = r"3";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::nat(3);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_not_nat() {
        let expr = r"\a1 . 1";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::lambda("a1", Ast::nat(1));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parens() {
        let expr = r"((\x. (x)))";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::lambda("x", Ast::var("x"));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_comments() {
        let expr = r"
            -- this is a comment
            let id = \x. x in -- another comment
            \id. id --end comment
        ";
        let parsed = tokenize_and_parse(expr);
        let expected = Ast::let_binding(
            "id",
            Ast::lambda("x", Ast::var("x")),
            Ast::lambda("id", Ast::var("id")),
        );
        assert_eq!(parsed, expected);
    }
}
