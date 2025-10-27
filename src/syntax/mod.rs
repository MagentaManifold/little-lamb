pub mod ast;
pub mod desugar;
pub mod expr;
pub mod term;

// Re-export main types
pub use ast::Ast;
pub use desugar::desugar;
pub use expr::Expr;
pub use term::{Term, TermInner};
