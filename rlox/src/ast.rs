use crate::{helpers::format_float, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LiteralValue {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

impl Eq for LiteralValue {}

impl std::hash::Hash for LiteralValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) enum Expr {
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(LiteralValue),
    Variable(Token),
    Assign {
        left: Token,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This(Token),
    Super {
        keyword: Token,
        method: Token,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionDecl {
    pub(crate) name: Token,
    pub(crate) parameters: Vec<Token>,
    pub(crate) body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        identifier: Token,
        expression: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function(FunctionDecl),
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Class {
        name: Token,
        superclass: Option<Token>,
        methods: Vec<FunctionDecl>,
    },
}

impl LiteralValue {
    pub(crate) fn to_string(&self) -> String {
        match self {
            LiteralValue::Number(num) => format_float(*num),
            LiteralValue::Str(s) => s.to_string(),
            LiteralValue::Bool(b) => b.to_string(),
            LiteralValue::Nil => String::from("nil"),
        }
    }
}

impl Expr {
    pub(crate) fn to_string(&self) -> String {
        match self {
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.lexeme, right.to_string())
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                left.to_string(),
                right.to_string()
            ),
            Expr::Grouping(expr) => format!("(group {})", expr.to_string()),
            Expr::Literal(literal_value) => literal_value.to_string(),
            Expr::Variable(token) => token.lexeme.to_string(),
            Expr::Assign {
                left: name,
                right: value,
            } => format!("(assign {} {})", name.lexeme, value.to_string()),
            Expr::Logical {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                left.to_string(),
                right.to_string()
            ),
            Expr::Call {
                callee: _,
                paren: _,
                arguments: _,
            } => todo!(),
            Expr::Get { object: _, name: _ } => todo!(),
            Expr::Set {
                object: _,
                name: _,
                value: _,
            } => todo!(),
            Expr::This(_token) => todo!(),
            Expr::Super { keyword: _, method: _method } => todo!(),
        }
    }
}
