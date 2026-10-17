use std::collections::HashMap;

use crate::interpreter::*;

pub struct Environment<'a> {
    pub symbols: HashMap<&'a [char], Value<'a>>,
    pub parent_key: Option<usize>,
    pub reachable: bool
}

impl<'a> Environment<'a> {
    pub fn new(parent_key: Option<usize>) -> Self {
        Self {
            symbols: HashMap::<&[char], Value>::new(),
            parent_key: parent_key,
            reachable: false
        }
    }
}

pub struct Context<'a> {
    pub environments: HashMap<usize, Environment<'a>>,
    pub environment_stack: Vec<usize>,
    pub next_id: usize
}

impl<'a> Context<'a> {

    pub fn new() -> Self {
        Self {
            environments: HashMap::<usize, Environment>::new(),
            environment_stack: Vec::<usize>::new(),
            next_id: 0
        }
    }

    pub fn push_new_environment(&mut self, parent_id: Option<usize>) {
        self.environments.insert (
            self.next_id,
            Environment::new(parent_id)
        );
        self.environment_stack.push(self.next_id);
        self.next_id += 1;

        if self.next_id % 512 == 0 {
            self.gc_collect();
        };

    }

    pub fn push_new_environment_auto(&mut self) {
        self.environments.insert (
            self.next_id,
            Environment::new(self.environment_stack.last().copied())
        );
        self.environment_stack.push(self.next_id);
        self.next_id += 1;

        if self.next_id % 512 == 0 {
            self.gc_collect();
        };

    }

    pub fn pop_environment(&mut self) {
        self.environment_stack.pop();
    }

    pub fn find_symbol(&self, identifier: &[char], lookup_hop_count: &mut Option<usize>) -> usize {

        let mut id = *self
            .environment_stack
            .last()
            .expect(&format!("Symbol {:#?} not found: interpreter context is empty", identifier));

        match lookup_hop_count {

            None => {
                let mut hop_count: usize = 0;
                loop {
                    let env = self
                        .environments
                        .get(&id)
                        .expect("Environment id does not exist");

                    if let Some(..) = env.symbols.get(identifier) {
                        *lookup_hop_count = Some(hop_count);
                        return id;
                    }

                    if let Some(parent_id) = env.parent_key {
                        id = parent_id;
                        hop_count += 1;
                    }
                    else {
                        *lookup_hop_count = Some(hop_count);
                        return id;
                    }
                }
            }
            Some(hop_count) => {
                let mut hops_left = *hop_count;
                loop {
                    let env = self
                        .environments
                        .get(&id)
                        .expect("Environment id does not exist");

                    if hops_left == 0 {
                        if let Some(..) = env.symbols.get(identifier) {
                            return id;
                        }
                        else {
                            panic!("Symbol {:#?} not found", identifier);
                        }
                    }

                    hops_left -= 1;
                    id = env.parent_key.expect(&format!("Symbol {:#?} not found", identifier));
                }
            }
        }
    }

    pub fn get_symbol_value(&self, identifier: &[char], lookup_hop_count: &mut Option<usize>) -> Value<'a> {

        let env_id = self.find_symbol(identifier, lookup_hop_count);
        let env = self.environments.get(&env_id).unwrap();

        if let Some(v) = env.symbols.get(identifier) {
            return v.clone();
        }
        else {
            panic!("Symbol not found")
        }
    }

    pub fn set_symbol_value(&mut self, identifier: &[char], value: Value<'a>, lookup_hop_count: &mut Option<usize>) {

        let env_id = self.find_symbol(identifier, lookup_hop_count);
        let env = self.environments.get_mut(&env_id).unwrap();

        if let Some(v) = env.symbols.get_mut(identifier) {
            *v = value;
            return;
        }
        else {
            panic!("Symbol not found")
        }
    }

    pub fn insert_symbol(&mut self, identifier: &'a [char], value: Value<'a>) {

        let id = *self
            .environment_stack
            .last()
            .expect("Interpreter context is empty");

        let env = self
            .environments
            .get_mut(&id)
            .expect("Environment id does not exist");

        if env.symbols.contains_key(identifier) && self.environment_stack.len() > 1 {
            panic!("Symbol already exists");
        }

        env.symbols.insert(identifier, value);
    }

    pub fn get_current_environment_id(&self) -> usize {

        return *self
            .environment_stack
            .last()
            .expect("Interpreter context is empty");
    }

    pub fn gc_collect(&mut self) {

        for (_, env) in &mut self.environments {
            env.reachable = false;
        }

        for env_chain_root_id in &mut self.environment_stack {

            let mut env_id = *env_chain_root_id;
            loop {
                let env = self.environments.get_mut(&env_id).unwrap();
                env.reachable = true;
                match env.parent_key {
                    Some(key) => { env_id = key; },
                    None => { break; }
                }
            }

        }

        self.environments.retain (
            |_, env|  {
                env.reachable
            }
        );
    }
}
