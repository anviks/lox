use std::collections::HashMap;

use crate::{
    ast::{Expr, FunctionDecl, Stmt},
    token::Token,
};

#[derive(Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
    Subclass,
}

pub(crate) struct Resolver {
    pub(crate) locals: HashMap<Expr, usize>,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

pub(crate) struct ResolveError {
    pub(crate) message: String,
}

impl Resolver {
    pub(crate) fn new() -> Self {
        Resolver {
            locals: HashMap::new(),
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), ResolveError> {
        if let Some(map) = self.scopes.last_mut()
            && let Some(_) = map.insert(name.clone().lexeme, false)
        {
            return Err(ResolveError {
                message: "Already a variable with this name in this scope.".to_string(),
            });
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(map) = self.scopes.last_mut() {
            map.insert(name.lexeme.clone(), true);
        }
    }

    pub(crate) fn resolve_statements(&mut self, stmts: &Vec<Stmt>) -> Result<(), ResolveError> {
        for stmt in stmts {
            self.resolve_statement(stmt)?;
        }
        Ok(())
    }

    fn resolve_local(&mut self, expr: &Expr, name: &str) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.locals.insert(expr.clone(), i);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        func: &FunctionDecl,
        func_type: FunctionType,
    ) -> Result<(), ResolveError> {
        let enclosing_func = self.current_function;
        self.current_function = func_type;
        self.begin_scope();

        for param in &func.parameters {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(&func.body)?;

        self.end_scope();
        self.current_function = enclosing_func;

        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Stmt) -> Result<(), ResolveError> {
        match stmt {
            Stmt::Expression(expr) => self.resolve_expression(expr),
            Stmt::Print(expr) => self.resolve_expression(expr),
            Stmt::Var {
                identifier,
                expression,
            } => {
                self.declare(identifier)?;
                if let Some(init) = expression {
                    self.resolve_expression(init)?;
                }
                self.define(identifier);
                Ok(())
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve_statements(stmts)?;
                self.end_scope();
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;
                match else_branch {
                    Some(branch) => self.resolve_statement(branch),
                    None => Ok(()),
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)
            }
            Stmt::Function(func) => {
                self.declare(&func.name)?;
                self.define(&func.name);
                self.resolve_function(func, FunctionType::Function)?;
                Ok(())
            }
            Stmt::Return { keyword: _, value } => match self.current_function {
                FunctionType::None => Err(ResolveError {
                    message: "Can't return from top-level code.".to_string(),
                }),
                FunctionType::Initializer if value.is_some() => Err(ResolveError {
                    message: "Can't return a value from an initializer.".to_string(),
                }),
                _ if let Some(ex) = value => self.resolve_expression(ex),
                _ => Ok(()),
            },
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(name)?;
                self.define(name);

                if let Some(sup) = superclass {
                    self.current_class = ClassType::Subclass;

                    if name.lexeme == sup.lexeme {
                        return Err(ResolveError {
                            message: "A class can't inherit from itself.".to_string(),
                        });
                    }

                    self.resolve_expression(&Expr::Variable(sup.clone()))?;

                    self.begin_scope();
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert("super".to_string(), true);
                }

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("this".to_string(), true);

                for method in methods {
                    let func_type = match method.name.lexeme.as_str() {
                        "init" => FunctionType::Initializer,
                        _ => FunctionType::Method,
                    };
                    self.resolve_function(method, func_type)?;
                }

                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;

                Ok(())
            }
        }
    }

    fn resolve_expression(&mut self, expr: &Expr) -> Result<(), ResolveError> {
        match expr {
            Expr::Unary { operator: _, right } => self.resolve_expression(right),
            Expr::Grouping(expr) => self.resolve_expression(expr),
            Expr::Literal(_literal_value) => Ok(()),
            Expr::Variable(token) => {
                if let Some(scope) = self.scopes.last()
                    && let Some(b) = scope.get(&token.lexeme)
                    && !*b
                {
                    Err(ResolveError {
                        message: "Can't read local variable in its own initializer.".to_string(),
                    })
                } else {
                    self.resolve_local(expr, &token.lexeme);
                    Ok(())
                }
            }
            Expr::Assign { left, right } => {
                self.resolve_expression(right)?;
                self.resolve_local(expr, &left.lexeme);
                Ok(())
            }
            Expr::Logical {
                left,
                operator: _,
                right,
            }
            | Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expression(callee)?;
                for arg in arguments {
                    self.resolve_expression(arg)?;
                }
                Ok(())
            }
            Expr::Get { object, name: _ } => self.resolve_expression(object),
            Expr::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expression(value)?;
                self.resolve_expression(object)
            }
            Expr::This(token) => match self.current_class {
                ClassType::Class | ClassType::Subclass => {
                    Ok(self.resolve_local(expr, &token.lexeme))
                }
                ClassType::None => Err(ResolveError {
                    message: "Can't use 'this' outside of a class.".to_string(),
                }),
            },
            Expr::Super { keyword, method: _ } => match self.current_class {
                ClassType::None => Err(ResolveError {
                    message: "Can't use 'super' outside of a class.".to_string(),
                }),
                ClassType::Class => Err(ResolveError {
                    message: "Can't use 'super' in a class with no superclass.".to_string(),
                }),
                ClassType::Subclass => {
                    self.resolve_local(expr, &keyword.lexeme);
                    Ok(())
                }
            },
        }
    }
}
