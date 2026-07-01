use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{LiteralValue, Stmt},
    environment::{Environment, EnvironmentExt},
    interpreter::Interpreter,
    token::Token,
};

pub(crate) struct RuntimeError {
    pub(crate) message: String,
}

pub(crate) enum ExecutionError {
    RuntimeError(RuntimeError),
    Return(LoxValue),
}

impl From<RuntimeError> for ExecutionError {
    fn from(value: RuntimeError) -> Self {
        Self::RuntimeError(value)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoxFunction {
    pub(crate) name: Token,
    pub(crate) parameters: Vec<Token>,
    pub(crate) body: Vec<Stmt>,
    pub(crate) closure: Rc<RefCell<Environment>>,
    pub(crate) is_initializer: bool,
}

impl LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError> {
        if args.len() != self.parameters.len() {
            return Err(RuntimeError {
                message: format!(
                    "Expected {} arguments but got {}.",
                    self.parameters.len(),
                    args.len()
                ),
            });
        }

        let environment = self.closure.child();

        for i in 0..args.len() {
            environment
                .borrow_mut()
                .define(self.parameters[i].lexeme.clone(), args[i].clone());
        }

        match interpreter.execute_block(&self.body, environment) {
            Err(ExecutionError::RuntimeError(err)) => return Err(err),
            _ if self.is_initializer => self.closure.get_at(0, "this"),
            Ok(_) => Ok(LoxValue::Nil),
            Err(ExecutionError::Return(ret)) => Ok(ret),
        }
    }

    pub(crate) fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> Self {
        let environment = self.closure.child();
        environment
            .borrow_mut()
            .define("this".to_string(), LoxValue::Instance(instance.clone()));
        LoxFunction {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            closure: environment,
            is_initializer: self.is_initializer,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoxClass {
    pub(crate) name: String,
    pub(crate) superclass: Option<Rc<LoxClass>>,
    pub(crate) methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    fn call(
        self: &Rc<LoxClass>,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError> {
        if args.len() != self.arity() {
            return Err(RuntimeError {
                message: format!(
                    "Expected {} arguments but got {}.",
                    self.arity(),
                    args.len()
                ),
            });
        }

        let instance = Rc::new(RefCell::new(LoxInstance {
            class: self.clone(),
            fields: HashMap::new(),
        }));

        if let Some(init) = self.find_method("init") {
            init.bind(instance.clone()).call(interpreter, args)?;
        }

        Ok(LoxValue::Instance(instance))
    }

    pub(crate) fn find_method(self: &Rc<LoxClass>, name: &str) -> Option<&LoxFunction> {
        self.methods.get(name).or_else(|| {
            self.superclass
                .as_ref()
                .and_then(|class| class.find_method(name))
        })
    }

    pub(crate) fn arity(self: &Rc<LoxClass>) -> usize {
        match self.find_method("init") {
            Some(init) => init.parameters.len(),
            None => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoxInstance {
    pub(crate) class: Rc<LoxClass>,
    pub(crate) fields: HashMap<String, LoxValue>,
}

impl LoxInstance {
    pub(crate) fn get(
        instance: &Rc<RefCell<LoxInstance>>,
        name: &Token,
    ) -> Result<LoxValue, RuntimeError> {
        instance
            .borrow()
            .fields
            .get(&name.lexeme)
            .cloned()
            .or_else(|| {
                instance
                    .borrow()
                    .class
                    .find_method(&name.lexeme)
                    .map(|m| LoxValue::Function(m.bind(instance.clone())))
            })
            .ok_or(RuntimeError {
                message: format!("Undefined property '{}'.", name.lexeme),
            })
    }

    pub(crate) fn set(&mut self, name: &Token, val: LoxValue) {
        self.fields.insert(name.lexeme.clone(), val);
    }
}

#[derive(Debug, Clone)]
pub(crate) enum LoxValue {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
    Function(LoxFunction),
    NativeFunction {
        name: String,
        arity: usize,
        func: fn(&mut Interpreter, Vec<LoxValue>) -> Result<LoxValue, RuntimeError>,
    },
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<LoxInstance>>),
}

impl From<LiteralValue> for LoxValue {
    fn from(literal: LiteralValue) -> Self {
        match literal {
            LiteralValue::Number(n) => LoxValue::Number(n),
            LiteralValue::Str(s) => LoxValue::Str(s),
            LiteralValue::Bool(b) => LoxValue::Bool(b),
            LiteralValue::Nil => LoxValue::Nil,
        }
    }
}

impl LoxValue {
    pub(crate) fn as_number(&self) -> Result<f64, RuntimeError> {
        if let LoxValue::Number(num) = self {
            Ok(*num)
        } else {
            return Err(RuntimeError {
                message: "Operand must be a number.".to_string(),
            });
        }
    }

    pub(crate) fn to_string(&self) -> String {
        match self {
            LoxValue::Number(n) => n.to_string(),
            LoxValue::Str(s) => s.to_string(),
            LoxValue::Bool(b) => b.to_string(),
            LoxValue::Nil => String::from("nil"),
            LoxValue::Function(lox_function) => {
                format!("<fn {}>", lox_function.name.lexeme)
            }
            LoxValue::Class(lox_class) => format!("{}", lox_class.name),
            LoxValue::NativeFunction {
                name,
                arity: _,
                func: _,
            } => {
                format!("<built-in function {}>", name)
            }
            LoxValue::Instance(lox_instance) => {
                format!("{} instance", lox_instance.borrow().class.name)
            }
        }
    }

    pub(crate) fn is_truthy(&self) -> bool {
        if let LoxValue::Bool(b) = self {
            return *b;
        }

        if let LoxValue::Nil = self {
            return false;
        }

        true
    }

    pub(crate) fn is_equal_to(&self, other: &Self) -> bool {
        match (self, other) {
            (LoxValue::Number(n1), LoxValue::Number(n2)) => n1 == n2,
            (LoxValue::Str(s1), LoxValue::Str(s2)) => s1 == s2,
            (LoxValue::Bool(b1), LoxValue::Bool(b2)) => b1 == b2,
            (LoxValue::Nil, LoxValue::Nil) => true,
            _ => false,
        }
    }

    pub(crate) fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError> {
        match self {
            LoxValue::Function(lox_function) => lox_function.call(interpreter, args),
            LoxValue::Class(lox_class) => lox_class.call(interpreter, args),
            LoxValue::NativeFunction {
                name: _,
                arity: _,
                func,
            } => func(interpreter, args),
            _ => Err(RuntimeError {
                message: "Not callable".to_string(),
            }),
        }
    }
}
