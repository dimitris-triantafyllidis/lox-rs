use std::collections::HashMap;

use crate::interpreter::*;

pub struct Environment<'a> {
    pub symbols: HashMap<&'a str, Value<'a>>,
    pub parent_key: Option<usize>,
    pub reachable: bool
}

impl<'a> Environment<'a> {
    pub fn new(parent_key: Option<usize>) -> Self {
        Self {
            symbols: HashMap::<&str, Value>::new(),
            parent_key: parent_key,
            reachable: false
        }
    }
}

pub struct Context<'a> {
    pub environments: HashMap<usize, Environment<'a>>,
    pub environment_stack: Vec<usize>,
    pub next_env_id: usize,
    pub instances: HashMap<usize, Instance<'a>>,
    pub next_instance_id: usize
}

impl<'a> Context<'a> {

    pub fn new() -> Self {
        Self {
            environments: HashMap::<usize, Environment>::new(),
            environment_stack: Vec::<usize>::new(),
            next_env_id: 0,
            instances: HashMap::<usize, Instance<'a>>::new(),
            next_instance_id: 0
        }
    }

    pub fn insert_instance(&mut self, instance: Instance<'a>) -> usize {
        self.instances.insert (
            self.next_instance_id,
            instance
        );
        self.next_instance_id += 1;
        return self.next_instance_id - 1;
    }

    pub fn push_new_environment(&mut self, parent_id: Option<usize>) {
        self.environments.insert (
            self.next_env_id,
            Environment::new(parent_id)
        );
        self.environment_stack.push(self.next_env_id);
        self.next_env_id += 1;

        if self.next_env_id % 512 == 0 {
            self.gc_collect();
        };

    }

    pub fn push_new_environment_auto(&mut self) {
        self.environments.insert (
            self.next_env_id,
            Environment::new(self.environment_stack.last().copied())
        );
        self.environment_stack.push(self.next_env_id);
        self.next_env_id += 1;

        if self.next_env_id % 512 == 0 {
            self.gc_collect();
        };

    }

    pub fn pop_environment(&mut self) {
        self.environment_stack.pop();
    }

    pub fn find_symbol(&self, identifier: &str, lookup_hop_count: &mut Option<usize>) -> usize {

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

    pub fn get_symbol_value(&self, identifier: &str, lookup_hop_count: &mut Option<usize>) -> Value<'a> {

        let env_id = self.find_symbol(identifier, lookup_hop_count);
        let env = self.environments.get(&env_id).unwrap();

        if let Some(v) = env.symbols.get(identifier) {
            return v.clone();
        }
        else {
            panic!("Symbol not found")
        }
    }

    pub fn set_symbol_value(&mut self, identifier: &str, value: Value<'a>, lookup_hop_count: &mut Option<usize>) {

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

    pub fn insert_symbol(&mut self, identifier: &'a str, value: Value<'a>) {

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

    pub fn gc_clear_flags(&mut self) {
        for (_, env) in &mut self.environments {
            env.reachable = false;
        }
    }

    pub fn gc_sweep(&mut self) {
        self.environments.retain (
            |_, env|  {
                env.reachable
            }
        );
    }

    pub fn gc_follow_and_mark(&mut self, env_id: usize) {

        let mut env_ids_to_mark_stack = Vec::<usize>::new();
        env_ids_to_mark_stack.push(env_id);

        while let Some(env_id) = env_ids_to_mark_stack.pop() {

            let Some(env) = self.environments.get_mut(&env_id) else {
                continue;
            };

            if env.reachable {
                continue;
            }

            env.reachable = true;

            let mut new_envs_to_visit = Vec::new();

            for value in env.symbols.values() {
                if let Value::Function(Function { closure_id, .. }) = value {
                    new_envs_to_visit.push(*closure_id);
                }
                else if let Value::Instance ( instance_id ) = value {
                    let instance = self.instances.get(instance_id).unwrap();
                }
                else if let Value::Class ( Class { methods, .. } ) = value {

                    for value in methods.values() {
                        if let Value::Function(Function { closure_id, .. } ) = value {
                            new_envs_to_visit.push(*closure_id);
                        }
                    }
                }
            }

            if let Some(parent_id) = env.parent_key {
                new_envs_to_visit.push(parent_id);
            }

            env_ids_to_mark_stack.extend(new_envs_to_visit);
        }

    }

    pub fn gc_collect(&mut self) {

        self.gc_clear_flags();

        let env_stack = self.environment_stack.clone();

        for env_chain_root_id in env_stack {
            let env_id = env_chain_root_id;
            self.gc_follow_and_mark(env_id);
        }

        self.gc_sweep();
    }
}
