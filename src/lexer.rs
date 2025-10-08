use chumsky::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub enum Token {
    Lambda,
    Dot,
    LParen,
    RParen,
    Let,
    In,
    Comma,
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
            Token::Comma => write!(f, ","),
            Token::Equal => write!(f, "="),
            Token::Ident(name) => write!(f, "{name}"),
        }
    }
}

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Token>, extra::Err<Rich<'src, char>>> {
    let lambda = just('\\').to(Token::Lambda).labelled("\\");
    let dot = just('.').to(Token::Dot).labelled(".");
    let lparen = just('(').to(Token::LParen).labelled("(");
    let rparen = just(')').to(Token::RParen).labelled(")");
    let let_kw = just("let").to(Token::Let).labelled("let");
    let in_kw = just("in").to(Token::In).labelled("in");
    let comma = just(',').to(Token::Comma).labelled(",");
    let equal = just('=').to(Token::Equal).labelled("=");
    let ident = text::ascii::ident()
        .map(|s: &str| Token::Ident(s.to_string()))
        .labelled("identifier");
    let comment = just("--")
        .ignore_then(any().and_is(just("\n").not()).repeated())
        .padded()
        .labelled("comment");

    choice((
        lambda, dot, lparen, rparen, let_kw, in_kw, equal, comma, ident,
    ))
    .padded_by(comment.repeated())
    .padded()
    .repeated()
    .at_least(1)
    .collect()
}
