pub mod common;
pub mod kn;
pub mod krivine;
pub mod substitution;

#[cfg(test)]
mod tests;

pub use common::{EvalError, de_bruijn};

pub use kn::eval as kn_eval;
pub use krivine::eval as krivine_eval;
pub use substitution::eval as substitution_eval;

// Default eval strategy is kn
pub use kn::eval;
