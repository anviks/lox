use std::time::SystemTime;

use crate::{
    interpreter::Interpreter,
    value::{LoxValue, RuntimeError},
};

pub(crate) fn clock(_: &mut Interpreter, _: Vec<LoxValue>) -> Result<LoxValue, RuntimeError> {
    Ok(LoxValue::Number(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            .floor(),
    ))
}
