use include_dir::{Dir, include_dir};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{
    ast::Expr,
    eval::{EvalError, desugar},
    lexer::tokenize,
    parser::parse,
};

static LIB_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/lib");

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("Failed to read {1}")]
    Io(#[source] std::io::Error, PathBuf),
    #[error("Failed to tokenize {1}")]
    Tokenize(#[source] crate::lexer::LexerError, PathBuf),
    #[error("Failed to parse {1}")]
    Parse(#[source] crate::parser::ParserError, PathBuf),
    #[error("Circular dependency detected: {0}")]
    CircularDependency(Cycle),
}

#[derive(Debug, Clone)]
pub struct Cycle(Vec<Resolution>);

impl Cycle {
    fn to_single_line_string(&self) -> String {
        self.0
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(" -> ")
    }

    fn to_multiline_string(&self) -> String {
        let mut result = String::new();
        for (i, r) in self.0.iter().enumerate() {
            if i == 0 {
                result.push_str(&format!("\n   {}", r));
            } else {
                result.push_str(&format!("\n-> {}", r));
            }
        }
        result
    }
}

impl Display for Cycle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        const MAX_LENGTH: usize = 60;
        let length = self.to_single_line_string().len();
        if length <= MAX_LENGTH {
            write!(f, "{}", self.to_single_line_string())
        } else {
            write!(f, "{}", self.to_multiline_string())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Resolution {
    Builtin(PathBuf),
    User(PathBuf),
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Resolution::Builtin(path) => write!(f, "{} (builtin)", path.display()),
            Resolution::User(path) => write!(f, "{}", path.display()),
        }
    }
}

impl Resolution {
    fn path(&self) -> &PathBuf {
        match self {
            Resolution::Builtin(path) => path,
            Resolution::User(path) => path,
        }
    }
}

pub struct Importer {
    stack: Vec<Resolution>,
}

impl Importer {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Import a module by its name, searching in the given file directory first,
    /// and then falling back to the built-in modules if not found. If `file_dir` is
    /// `None`, only built-in modules are searched.
    pub fn import(&mut self, module: &str, file_dir: Option<&Path>) -> Result<Expr, EvalError> {
        let relative_path = PathBuf::from(format!("{}.lil", module));
        let (src, resolution) = match file_dir {
            Some(dir) => std::fs::read_to_string(dir.join(&relative_path))
                .map(|s| (s, Resolution::User(dir.join(&relative_path))))
                .or_else(|err| {
                    get_builtin_module_src(&relative_path)
                        .map(|s| (s, Resolution::Builtin(PathBuf::from(&relative_path))))
                        .map_err(|_| {
                            EvalError::Import(ImportError::Io(err, dir.join(&relative_path)))
                        })
                })?,
            None => (
                get_builtin_module_src(&relative_path).map_err(EvalError::from)?,
                Resolution::Builtin(PathBuf::from(&relative_path)),
            ),
        };
        if self.stack.contains(&resolution) {
            let cycle = Cycle(
                self.stack
                    .iter()
                    .skip_while(|&path| *path != resolution)
                    .cloned()
                    .chain(std::iter::once(resolution.clone()))
                    .collect(),
            );
            return Err(EvalError::Import(ImportError::CircularDependency(cycle)));
        }
        self.stack.push(resolution.clone());
        let tokens = tokenize(&src)
            .map_err(|err| ImportError::Tokenize(err, resolution.path().to_path_buf()))?;
        let ast = parse(&tokens)
            .map_err(|err| ImportError::Parse(err, resolution.path().to_path_buf()))?;
        let file_dir = match &resolution {
            Resolution::Builtin(_) => None,
            Resolution::User(_) => file_dir,
        };
        let expr = desugar(ast, self, file_dir)?;
        self.stack.pop();
        Ok(expr)
    }
}

fn get_builtin_module_src(file: &Path) -> Result<String, ImportError> {
    let path = file.strip_prefix("/").unwrap_or(file);
    LIB_DIR
        .get_file(path)
        .and_then(|f| f.contents_utf8().map(String::from))
        .ok_or_else(|| {
            ImportError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Builtin module file not found: {}", path.display()),
                ),
                path.to_path_buf(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_builtin() {
        let mut importer = Importer::new();
        let expr = importer.import("I", Some(Path::new("."))).unwrap();
        let expected = Expr::lambda("x", Expr::var("x"));
        assert_eq!(expr, expected);
    }
}
