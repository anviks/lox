use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{Expr, FunctionDecl, Stmt},
    environment::{Environment, EnvironmentExt},
    natives::clock,
    token::{Token, TokenType},
    value::{ExecutionError, LoxClass, LoxFunction, LoxInstance, LoxValue, RuntimeError},
};

pub(crate) struct Interpreter {
    pub(crate) globals: Rc<RefCell<Environment>>,
    pub(crate) environment: Rc<RefCell<Environment>>,
    pub(crate) locals: HashMap<Expr, usize>,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        let mut env = Environment::new();
        let clock_fn = LoxValue::NativeFunction {
            name: "clock".to_string(),
            arity: 0,
            func: clock,
        };
        env.define("clock".to_string(), clock_fn);

        let environment = Rc::new(RefCell::new(env));
        Self {
            globals: environment.clone(),
            environment,
            locals: HashMap::new(),
        }
    }

    pub(crate) fn look_up_variable(
        &self,
        name: &Token,
        expr: &Expr,
    ) -> Result<LoxValue, RuntimeError> {
        match self.locals.get(expr) {
            Some(distance) => self.environment.get_at(*distance, &name.lexeme),
            None => Ok(self.globals.borrow().get(name.clone())?.clone()),
        }
    }

    pub(crate) fn evaluate(&mut self, expr: &Expr) -> Result<LoxValue, RuntimeError> {
        match expr {
            Expr::Unary { operator, right } => {
                let r = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => Ok(LoxValue::Number(-r.as_number()?)),
                    TokenType::Bang => Ok(LoxValue::Bool(!r.is_truthy())),
                    _ => Ok(LoxValue::Nil),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;

                match &operator.token_type {
                    TokenType::Slash => Ok(LoxValue::Number(l.as_number()? / r.as_number()?)),
                    TokenType::Star => Ok(LoxValue::Number(l.as_number()? * r.as_number()?)),
                    TokenType::Minus => Ok(LoxValue::Number(l.as_number()? - r.as_number()?)),
                    TokenType::Plus => match (l, r) {
                        (LoxValue::Number(l_num), LoxValue::Number(r_num)) => {
                            Ok(LoxValue::Number(l_num + r_num))
                        }
                        (LoxValue::Str(l_str), LoxValue::Str(r_str)) => {
                            Ok(LoxValue::Str(l_str + &r_str))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be two numbers or two strings.".to_string(),
                        }),
                    },
                    TokenType::Less => Ok(LoxValue::Bool(l.as_number()? < r.as_number()?)),
                    TokenType::LessEqual => Ok(LoxValue::Bool(l.as_number()? <= r.as_number()?)),
                    TokenType::Greater => Ok(LoxValue::Bool(l.as_number()? > r.as_number()?)),
                    TokenType::GreaterEqual => Ok(LoxValue::Bool(l.as_number()? >= r.as_number()?)),
                    TokenType::EqualEqual => Ok(LoxValue::Bool(l.is_equal_to(&r))),
                    TokenType::BangEqual => Ok(LoxValue::Bool(!l.is_equal_to(&r))),
                    typ => Err(RuntimeError {
                        message: format!("Bad operator for binary expression: {}", typ.to_string()),
                    }),
                }
            }
            Expr::Grouping(ex) => Ok(self.evaluate(ex)?),
            Expr::Literal(literal_value) => Ok(literal_value.clone().into()),
            Expr::Variable(token) => self.look_up_variable(token, expr),
            Expr::Assign { left, right } => {
                let value = self.evaluate(right)?;

                match self.locals.get(expr) {
                    Some(distance) => {
                        self.environment
                            .assign_at(*distance, &left.lexeme, value.clone())
                    }
                    None => self
                        .globals
                        .borrow_mut()
                        .assign(left.clone(), value.clone())?,
                }

                Ok(value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let l = self.evaluate(left)?;

                match operator.token_type {
                    TokenType::And if !l.is_truthy() => Ok(l),
                    TokenType::Or if l.is_truthy() => Ok(l),
                    _ => self.evaluate(right),
                }
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                let callable = self.evaluate(callee)?;

                let mut args = vec![];
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }

                callable.call(self, args)
            }
            Expr::Get { object, name } => {
                let obj = self.evaluate(object)?;
                match obj {
                    LoxValue::Instance(instance) => Ok(LoxInstance::get(&instance, name)?),
                    _ => Err(RuntimeError {
                        message: "Only instances have properties.".to_string(),
                    }),
                }
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                let obj = self.evaluate(object)?;
                match obj {
                    LoxValue::Instance(instance) => {
                        let val = self.evaluate(value)?;
                        instance.borrow_mut().set(name, val.clone());
                        Ok(val)
                    }
                    _ => Err(RuntimeError {
                        message: "Only instances have fields.".to_string(),
                    }),
                }
            }
            Expr::This(token) => self.look_up_variable(token, expr),
            Expr::Super { keyword: _, method } => {
                let distance = self.locals.get(expr);

                let LoxValue::Class(superclass) =
                    self.environment.get_at(*distance.unwrap(), "super")?
                else {
                    panic!("Expected 'super' to be a class")
                };

                let LoxValue::Instance(object) =
                    // Little hacky
                    self.environment.get_at(*distance.unwrap() - 1, "this")?
                else {
                    panic!("Expected 'this' to be an instance")
                };

                superclass
                    .find_method(&method.lexeme)
                    .map(|fun| LoxValue::Function(fun.bind(object)))
                    .ok_or_else(|| RuntimeError {
                        message: format!("Undefined property '{}'.", method.lexeme),
                    })
            }
        }
    }

    pub(crate) fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), ExecutionError> {
        let previous = self.environment.clone();
        self.environment = environment;

        let result = statements.iter().try_for_each(|stmt| self.execute(&stmt));
        self.environment = previous;

        result
    }

    pub(crate) fn execute(&mut self, stmt: &Stmt) -> Result<(), ExecutionError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let val = self.evaluate(expr)?;
                println!("{}", val.to_string());
                Ok(())
            }
            Stmt::Var {
                identifier,
                expression,
            } => match expression {
                Some(expr) => {
                    let value = self.evaluate(expr)?;
                    Ok(self
                        .environment
                        .borrow_mut()
                        .define(identifier.lexeme.clone(), value))
                }
                None => Ok(self
                    .environment
                    .borrow_mut()
                    .define(identifier.lexeme.clone(), LoxValue::Nil)),
            },
            Stmt::Block(stmts) => self.execute_block(stmts, self.environment.child()),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(stmt) = else_branch {
                    self.execute(stmt)?;
                }

                Ok(())
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
                Ok(())
            }
            Stmt::Function(FunctionDecl {
                name,
                parameters,
                body,
            }) => {
                let function = LoxValue::Function(LoxFunction {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    body: body.clone(),
                    closure: self.environment.clone(),
                    is_initializer: false,
                });
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), function);
                Ok(())
            }
            Stmt::Return { keyword: _, value } => {
                let return_value = match value {
                    Some(ex) => self.evaluate(ex)?,
                    None => LoxValue::Nil,
                };
                Err(ExecutionError::Return(return_value))
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let supr = if let Some(sup) = superclass {
                    let value = self.evaluate(&Expr::Variable(sup.clone()))?;
                    match value {
                        LoxValue::Class(lox_class) => Some(lox_class),
                        _ => {
                            return Err(ExecutionError::RuntimeError(RuntimeError {
                                message: "Superclass must be a class".to_string(),
                            }));
                        }
                    }
                } else {
                    None
                };

                let mut env = self.environment.clone();
                env.borrow_mut().define(name.lexeme.clone(), LoxValue::Nil);

                if let Some(sup) = &supr {
                    env = env.child();
                    env.borrow_mut()
                        .define("super".to_string(), LoxValue::Class(sup.clone()));
                }

                let mut meths = HashMap::new();
                for method in methods {
                    let function = LoxFunction {
                        name: method.name.clone(),
                        parameters: method.parameters.clone(),
                        body: method.body.clone(),
                        closure: env.clone(),
                        is_initializer: method.name.lexeme == "init",
                    };
                    meths.insert(method.name.lexeme.clone(), function);
                }

                if supr.is_some() {
                    let temp_env = env.borrow().parent.clone().unwrap();
                    env = temp_env;
                }

                let class = LoxClass {
                    name: name.lexeme.clone(),
                    superclass: supr,
                    methods: meths,
                };

                env.borrow_mut()
                    .assign(name.clone(), LoxValue::Class(Rc::new(class)))?;
                Ok(())
            }
        }
    }

    pub(crate) fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in statements {
            match self.execute(stmt) {
                Ok(()) => (),
                Err(ExecutionError::RuntimeError(err)) => return Err(err),
                Err(ExecutionError::Return(_)) => {
                    return Err(RuntimeError {
                        message: "Can't return from top-level code.".to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}
