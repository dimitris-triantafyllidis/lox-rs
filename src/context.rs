use std::collections::HashMap;

use crate::interpreter::*;

pub struct Environment {
    pub symbols: HashMap<String, Value>,
    pub parent_key: Option<usize>
}

impl Environment {
    pub fn new(parent_key: Option<usize>) -> Self {
        Self {
            symbols: HashMap::<String, Value>::new(),
            parent_key: parent_key
        }
    }
}

pub struct Context {
    pub environments: HashMap<usize, Environment>,
    pub environment_stack: Vec<usize>,
    pub next_id: usize
}

impl Context {

    pub fn new() -> Self {
        Self {
            environments: HashMap::<usize, Environment>::new(),
            environment_stack: Vec::<usize>::new(),
            next_id: 0
        }
    }

    pub fn push_new_environment(self: &mut Self, parent_id: Option<usize>) {
        self.environments.insert (
            self.next_id,
            Environment::new(parent_id)
        );
        self.environment_stack.push(self.next_id);
        self.next_id += 1;
    }

    pub fn push_new_environment_auto(self: &mut Self) {
        self.environments.insert (
            self.next_id,
            Environment::new(self.environment_stack.last().copied())
        );
        self.environment_stack.push(self.next_id);
        self.next_id += 1;
    }

    pub fn pop_environment(self: &mut Self) {
        self.environment_stack.pop();
    }

    pub fn get_symbol_value(self: &Self, identifier: &String) -> Value {

        let mut id = *self
            .environment_stack
            .last()
            .expect(&format!("Symbol {identifier} not found: interpreter context is empty"));

        loop {
            let env = self
                .environments
                .get(&id)
                .expect("Environment id does not exist");

            if let Some(value) = env.symbols.get(identifier) {
                return value.clone();
            }

            id = env.parent_key.expect(&format!("Symbol {identifier} not found"));
        }
    }

    pub fn set_symbol_value(&mut self, identifier: &String, value: Value) {

        let mut id = *self
            .environment_stack
            .last()
            .expect(&format!("Symbol {identifier} not found: interpreter context is empty"));

        loop {
            let env = self
                .environments
                .get_mut(&id)
                .expect("Environment id does not exist");

            if let Some(v) = env.symbols.get_mut(identifier) {
                *v = value;
                return;
            }

            id = env.parent_key.expect(&format!("Symbol {identifier} not found"));
        }
    }

    pub fn insert_symbol(&mut self, identifier: &String, value: Value) {

        let id = *self
            .environment_stack
            .last()
            .expect("Interpreter context is empty");

        let env = self
            .environments
            .get_mut(&id)
            .expect("Environment id does not exist");

        if env.symbols.contains_key(identifier) {
            panic!("Symbol already exists");
        }

        env.symbols.insert(identifier.clone(), value);
    }

    pub fn get_current_environment_id(self: &Self) -> usize {

        return *self
            .environment_stack
            .last()
            .expect("Interpreter context is empty");

    }

}
