use clover::{NativeModel, NativeModelInstance, Object, Reference, State};
use clover::debug::{Position, RuntimeError};
use clover::helper::{ensure_parameters_length, make_reference};
use crate::engine::graphics::{Color, Graphics};

impl NativeModel for Color {
    fn call(&mut self, _state: &mut State, parameters: &[Object]) -> Result<Object, RuntimeError> {
        let r = if parameters.len() > 0 { parameters[0].integer_value()? } else { 0 } as u8;
        let g = if parameters.len() > 1 { parameters[1].integer_value()? } else { 0 } as u8;
        let b = if parameters.len() > 2 { parameters[2].integer_value()? } else { 0 } as u8;
        let a = if parameters.len() > 3 { parameters[3].integer_value()? } else { 255 } as u8;

        Ok(Object::NativeInstance(make_reference(Color::new(r, g, b, a))))
    }
}

impl From<Reference<dyn NativeModelInstance>> for Color {
    fn from(source: Reference<dyn NativeModelInstance>) -> Self {
        let r = source.borrow().raw_get_integer("r").unwrap_or(0) as u8;
        let g = source.borrow().raw_get_integer("g").unwrap_or(0) as u8;
        let b = source.borrow().raw_get_integer("b").unwrap_or(0) as u8;
        let a = source.borrow().raw_get_integer("a").unwrap_or(255) as u8;
        Color::new(r, g, b, a)
    }
}

impl NativeModelInstance for Color {
    fn index_get(&self, this: Reference<dyn NativeModelInstance>, index: &Object) -> Result<Object, RuntimeError> {
        self.instance_get(this, index.string_value()?.as_str())
    }

    fn index_set(&mut self, this: Reference<dyn NativeModelInstance>, index: &Object, value: Object) -> Result<(), RuntimeError> {
        self.instance_set(this, index.string_value()?.as_str(), value)
    }

    fn instance_get(&self, this: Reference<dyn NativeModelInstance>, key: &str) -> Result<Object, RuntimeError> {
        match key {
            "r" => Ok(Object::Integer(self.r as i64)),
            "g" => Ok(Object::Integer(self.g as i64)),
            "b" => Ok(Object::Integer(self.b as i64)),
            "a" => Ok(Object::Integer(self.a as i64)),
            "blend" | "alpha_blend" => Ok(Object::InstanceNativeFunction(this, key.to_string())),
            _ => Err(RuntimeError::new("index not exists", Position::none()))
        }
    }

    fn instance_set(&mut self, _this: Reference<dyn NativeModelInstance>, key: &str, value: Object) -> Result<(), RuntimeError> {
        match key {
            "r" => self.r = value.integer_value()? as u8,
            "g" => self.g = value.integer_value()? as u8,
            "b" => self.b = value.integer_value()? as u8,
            "a" => self.a = value.integer_value()? as u8,
            _ => return Err(RuntimeError::new(&format!("can not set {}", key), Position::none()))
        };
        Ok(())
    }

    fn call(&mut self, this: Reference<dyn NativeModelInstance>, state: &mut State, key: &str, parameters: &[Object]) -> Result<Object, RuntimeError> {
        match key {
            "blend" => {
                ensure_parameters_length(parameters, 1)?;
                let color: Color = Color::from(parameters[0].native_instance_value()?);
                Ok(Object::NativeInstance(make_reference(self.blend(&color))))
            },
            "alpha_blend" => {
                ensure_parameters_length(parameters, 1)?;
                let color: Color = Color::from(parameters[0].native_instance_value()?);
                let alpha = parameters[1].float_value()?;
                Ok(Object::NativeInstance(make_reference(self.alpha_blend(&color,alpha))))
            }
            _ => Err(RuntimeError::new(&format!("can not call {}", key), state.last_position()))
        }
    }

    fn raw_get_integer(&self, key: &str) -> Option<i64> {
        match key {
            "r" => Some(self.r as i64),
            "g" => Some(self.g as i64),
            "b" => Some(self.b as i64),
            "a" => Some(self.a as i64),
            _ => None
        }
    }
}


impl NativeModelInstance for Graphics {
    fn index_get(&self, this: Reference<dyn NativeModelInstance>, index: &Object) -> Result<Object, RuntimeError> {
        todo!()
    }

    fn index_set(&mut self, this: Reference<dyn NativeModelInstance>, index: &Object, value: Object) -> Result<(), RuntimeError> {
        todo!()
    }

    fn instance_get(&self, this: Reference<dyn NativeModelInstance>, key: &str) -> Result<Object, RuntimeError> {
        todo!()
    }

    fn instance_set(&mut self, this: Reference<dyn NativeModelInstance>, key: &str, value: Object) -> Result<(), RuntimeError> {
        todo!()
    }

    fn call(&mut self, this: Reference<dyn NativeModelInstance>, state: &mut State, key: &str, parameters: &[Object]) -> Result<Object, RuntimeError> {
        todo!()
    }
}