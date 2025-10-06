use crate::ast::Expr;
use chumsky::prelude::*;

#[derive(Clone, PartialEq, Eq)]
enum Token {
    Lambda,
    Dot,
    LParen,
    RParen,
    Let,
    In,
    Equal,
    Ident(String),
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Lambda => write!(f, "\\"),
            Token::Dot => write!(f, "."),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Let => write!(f, "let"),
            Token::In => write!(f, "in"),
            Token::Equal => write!(f, "="),
            Token::Ident(name) => write!(f, "{name}"),
        }
    }
}

fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Token>, extra::Err<Rich<'src, char>>> {
    let lambda = just('\\').to(Token::Lambda).labelled("\\");
    let dot = just('.').to(Token::Dot).labelled(".");
    let lparen = just('(').to(Token::LParen).labelled("(");
    let rparen = just(')').to(Token::RParen).labelled(")");
    let let_kw = just("let").to(Token::Let).labelled("let");
    let in_kw = just("in").to(Token::In).labelled("in");
    let equal = just('=').to(Token::Equal).labelled("=");
    let ident = text::ascii::ident()
        .map(|s: &str| Token::Ident(s.to_string()))
        .labelled("identifier");
    let comment = just("--")
        .ignore_then(any().and_is(just("\n").not()).repeated())
        .padded()
        .labelled("comment");

    choice((lambda, dot, lparen, rparen, let_kw, in_kw, equal, ident))
        .padded_by(comment.repeated())
        .padded()
        .repeated()
        .at_least(1)
        .collect()
}

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
            .ignore_then(ident.clone())
            .then_ignore(just(Token::Dot))
            .then(expr.clone())
            .map(|(param, body): (String, Expr)| Expr::lambda(param, body))
            .labelled("lambda");

        let let_binding = just(Token::Let)
            .ignore_then(ident.clone())
            .then_ignore(just(Token::Equal))
            .then(expr.clone())
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .map(|((name, value), body): ((String, Expr), Expr)| {
                Expr::apply(Expr::lambda(name, body), value)
            });

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
        let expr = r"\x . \y. x y";
        let parsed = parse(expr);
        let expected = Expr::lambda(
            "x",
            Expr::lambda("y", Expr::apply(Expr::var("x"), Expr::var("y"))),
        );
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_deep_application() {
        let expr = r"\x . \y . \z . x y z";
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
            \id . id
        ";
        let parsed = parse(expr);
        let expected = Expr::apply(
            Expr::lambda("id", Expr::lambda("id", Expr::var("id"))),
            Expr::lambda("x", Expr::var("x")),
        );
        assert_eq!(parsed.unwrap(), expected);
    }

    #[test]
    fn test_comments() {
        let expr = r"
            -- this is a comment
            let id = \x. x in -- another comment
            \id . id --end comment
        ";
        let parsed = parse(expr);
        let expected = Expr::apply(
            Expr::lambda("id", Expr::lambda("id", Expr::var("id"))),
            Expr::lambda("x", Expr::var("x")),
        );
        assert_eq!(parsed.unwrap(), expected);
    }
}
