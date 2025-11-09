#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    Var(String),
    Nat(usize),
    Lambda {
        param: String,
        body: Box<Ast>,
    },
    Apply {
        func: Box<Ast>,
        arg: Box<Ast>,
    },
    Let {
        name: String,
        value: Box<Ast>,
        body: Box<Ast>,
    },
    Letrec {
        name: String,
        value: Box<Ast>,
        body: Box<Ast>,
    },
    Import {
        module: String,
        name: String,
        body: Box<Ast>,
    },
}

impl Ast {
    /// Create a new variable AST node
    pub fn var(name: impl Into<String>) -> Self {
        Ast::Var(name.into())
    }

    /// Create a new natural number AST node
    pub fn nat(value: usize) -> Self {
        Ast::Nat(value)
    }

    /// Create a new lambda AST node
    pub fn lambda(param: impl Into<String>, body: Ast) -> Self {
        Ast::Lambda {
            param: param.into(),
            body: Box::new(body),
        }
    }

    /// Create a new application AST node
    pub fn apply(func: Ast, arg: Ast) -> Self {
        Ast::Apply {
            func: Box::new(func),
            arg: Box::new(arg),
        }
    }

    /// Create a new let binding AST node
    pub fn let_binding(name: impl Into<String>, value: Ast, body: Ast) -> Self {
        Ast::Let {
            name: name.into(),
            value: Box::new(value),
            body: Box::new(body),
        }
    }

    /// Create a new letrec binding AST node
    pub fn letrec_binding(name: impl Into<String>, value: Ast, body: Ast) -> Self {
        Ast::Letrec {
            name: name.into(),
            value: Box::new(value),
            body: Box::new(body),
        }
    }

    /// Create a new import AST node
    pub fn import(module: impl Into<String>, name: impl Into<String>, body: Ast) -> Self {
        Ast::Import {
            module: module.into(),
            name: name.into(),
            body: Box::new(body),
        }
    }
}
