use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    token::Token,
    value::{LoxValue, RuntimeError},
};

#[derive(Debug)]
pub(crate) struct Environment {
    pub(crate) variables: HashMap<String, LoxValue>,
    pub(crate) parent: Option<Rc<RefCell<Environment>>>,
}

pub(crate) trait EnvironmentExt {
    fn child(&self) -> Rc<RefCell<Environment>>;
    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>>;
    fn get_at(&self, distance: usize, name: &str) -> Result<LoxValue, RuntimeError>;
    fn assign_at(&self, distance: usize, name: &str, value: LoxValue);
}

impl EnvironmentExt for Rc<RefCell<Environment>> {
    fn child(&self) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new({
            let parent = self.clone();
            Environment {
                variables: HashMap::new(),
                parent: Some(parent),
            }
        }))
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment = self.clone();

        for _ in 0..distance {
            let x = environment.borrow().parent.clone().unwrap();
            environment = x;
        }

        environment
    }

    fn get_at(&self, distance: usize, name: &str) -> Result<LoxValue, RuntimeError> {
        self.ancestor(distance)
            .borrow()
            .variables
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError {
                message: format!("Undefined variable '{}'.", name),
            })
    }

    fn assign_at(&self, distance: usize, name: &str, value: LoxValue) {
        let env = self.ancestor(distance);
        env.borrow_mut().variables.insert(name.to_string(), value);
    }
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub(crate) fn define(&mut self, name: String, value: LoxValue) {
        self.variables.insert(name, value);
    }

    pub(crate) fn assign(&mut self, name: Token, value: LoxValue) -> Result<(), RuntimeError> {
        if self.variables.contains_key(&name.lexeme) {
            self.variables.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        match &mut self.parent {
            Some(parent) => parent.borrow_mut().assign(name, value),
            None => Err(RuntimeError {
                message: format!("Undefined variable '{}'.", name.lexeme),
            }),
        }
    }

    pub(crate) fn get(&self, name: Token) -> Result<LoxValue, RuntimeError> {
        let value = self.variables.get(&name.lexeme);
        match value {
            Some(val) => Ok(val.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().get(name),
                None => Err(RuntimeError {
                    message: format!("Undefined variable '{}'.", name.lexeme),
                }),
            },
        }
    }
}
