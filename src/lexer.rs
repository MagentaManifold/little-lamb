use chumsky::{prelude::*, text::keyword};
use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub enum Token {
    Lambda,
    Dot,
    LParen,
    RParen,
    Let,
    Import,
    From,
    As,
    In,
    Comma,
    Equal,
    Ident(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Lambda => write!(f, "\\"),
            Token::Dot => write!(f, "."),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Let => write!(f, "let"),
            Token::Import => write!(f, "import"),
            Token::From => write!(f, "from"),
            Token::As => write!(f, "as"),
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
    let let_kw = keyword("let").to(Token::Let).labelled("let");
    let import_kw = keyword("import").to(Token::Import).labelled("import");
    let from_kw = keyword("from").to(Token::From).labelled("from");
    let as_kw = keyword("as").to(Token::As).labelled("as");
    let in_kw = keyword("in").to(Token::In).labelled("in");
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
        lambda, dot, lparen, rparen, let_kw, import_kw, from_kw, as_kw, in_kw, equal, comma, ident,
    ))
    .padded_by(comment.repeated())
    .padded()
    .repeated()
    .at_least(1)
    .collect()
}

#[derive(Debug, Error)]
#[error("Lexer errors: {0:?}")]
pub struct LexerError(Vec<String>);

impl LexerError {
    fn new(errors: Vec<Rich<'_, char>>) -> Self {
        LexerError(errors.into_iter().map(|e| e.to_string()).collect())
    }
}

pub fn tokenize<'src>(src: &str) -> Result<Vec<Token>, LexerError> {
    lexer().parse(src).into_result().map_err(LexerError::new)
}
