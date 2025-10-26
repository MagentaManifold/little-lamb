pub mod common;
pub mod krivine;
pub mod substitution;

#[cfg(test)]
mod tests;

pub use common::{EvalError, de_bruijn};

pub use krivine::eval as krivine_eval;
pub use substitution::eval as substitution_eval;

// Default eval strategy is Krivine
pub use krivine::eval;
