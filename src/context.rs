use std::collections::HashMap;

use crate::interpreter::*;

pub struct Environment<'a> {
    pub symbols: Vec<(&'a str, Value)>,
    pub parent_key: Option<usize>,
    pub reachable: bool
}

impl<'a> Environment<'a> {
    pub fn new(parent_key: Option<usize>) -> Self {
        Self {
            symbols: Vec::<(&str, Value)>::new(),
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
    pub next_instance_id: usize,
    pub classes: Vec<Class<'a>>,
    pub functions: Vec<Function<'a>>
}

impl<'a> Context<'a> {

    pub fn new() -> Self {
        Self {
            environments: HashMap::<usize, Environment>::new(),
            environment_stack: Vec::<usize>::new(),
            next_env_id: 0,
            instances: HashMap::<usize, Instance<'a>>::new(),
            next_instance_id: 0,
            classes: Vec::<Class>::new(),
            functions: Vec::<Function<'a>>::new()
        }
    }

    pub fn insert_class(&mut self, class: Class<'a>) -> usize {
        self.classes.push (
            class
        );
        return self.classes.len() - 1;
    }

    pub fn insert_function(&mut self, function: Function<'a>) -> usize {
        self.functions.push (
            function
        );
        return self.functions.len() - 1;
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

    }

    pub fn push_new_environment_auto(&mut self) {
        self.environments.insert (
            self.next_env_id,
            Environment::new(self.environment_stack.last().copied())
        );
        self.environment_stack.push(self.next_env_id);
        self.next_env_id += 1;
    }

    pub fn pop_environment(&mut self) {

        self.environment_stack.pop();
    }

    pub fn find_symbol(&self, identifier: &str, lookup_hop_count: &mut Option<usize>, env_slot_idx: &mut Option<usize>) -> usize {

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

                    if let Some(idx) = env.symbols.iter().position(|(id, _)| { return *id == identifier; })  {
                        *lookup_hop_count = Some(hop_count);
                        *env_slot_idx = Some(idx);
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
                        let Some(..) = env.symbols.iter().position(|(id, _)| { return *id == identifier; }) else {
                            panic!("Undefined variable '{}'.", identifier);
                        };
                        return id;
                    }

                    hops_left -= 1;
                    id = env.parent_key.expect(&format!("Symbol {:#?} not found", identifier));
                }
            }
        }
    }

    pub fn get_symbol_value(&self, identifier: &str, lookup_hop_count: &mut Option<usize>, env_slot_idx: &mut Option<usize>) -> Value {

        let env_id = self.find_symbol(identifier, lookup_hop_count, env_slot_idx);
        let env = self.environments.get(&env_id).unwrap();

        match env_slot_idx {

            None => {
                let Some(idx) = env.symbols.iter().position(|(id, _)| { return *id == identifier; }) else {
                    if identifier == "super" {
                        panic!("Can't use 'super' in a class with no superclass.");
                    }
                    else {
                        panic!("Symbol not found");
                    }
                };
                return env.symbols[idx].1.clone();
            },
            Some(idx) => {
                return env.symbols[*idx].1.clone();
            }
        }
    }

    pub fn set_symbol_value(&mut self, identifier: &str, value: Value, lookup_hop_count: &mut Option<usize>, env_slot_idx: &mut Option<usize>) {

        let env_id = self.find_symbol(identifier, lookup_hop_count, env_slot_idx);
        let env = self.environments.get_mut(&env_id).unwrap();

        match env_slot_idx {
            None => {
                let Some(idx) = env.symbols.iter().position(|(id, _)| { return *id == identifier; }) else {
                    if identifier == "super" {
                        panic!("Can't use 'super' in a class with no superclass.");
                    }
                    else {
                        panic!("Symbol not found");
                    }
                };
                env.symbols[idx].1 = value;
                *env_slot_idx = Some(idx);
                return;
            },
            Some(idx) => {
                env.symbols[*idx].1 = value;
            }
        }
    }

    pub fn insert_symbol(&mut self, identifier: &'a str, value: Value) {

        let id = *self
            .environment_stack
            .last()
            .expect("Interpreter context is empty");

        let env = self
            .environments
            .get_mut(&id)
            .expect("Environment id does not exist");

        if let Some(idx) = env.symbols.iter().position(|(id, _)| { return *id == identifier; }) {
            if self.environment_stack.len() > 1 {
                panic!("{identifier}: Already a variable with this name in this scope.");
            }
            else {
                env.symbols[idx].1 = value;
            }
        }
        else {
            env.symbols.push((identifier, value));
        }

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
        for (_, instance) in &mut self.instances {
            instance.reachable = false;
        }
    }

    pub fn gc_sweep(&mut self) {
        self.environments.retain (
            |_, env|  {
                env.reachable
            }
        );
        self.instances.retain (
            |_, instance|  {
                instance.reachable
            }
        );
    }

    pub fn gc_follow_and_mark(&mut self, env_id: usize) {

        let mut env_ids_to_mark_stack = Vec::<usize>::new();
        let mut instance_ids_to_mark_stack = Vec::<usize>::new();

        env_ids_to_mark_stack.push(env_id);

        while !env_ids_to_mark_stack.is_empty() || !instance_ids_to_mark_stack.is_empty() {

            while let Some(env_id) = env_ids_to_mark_stack.pop() {

                let Some(env) = self.environments.get_mut(&env_id) else {
                    continue;
                };

                if env.reachable {
                    continue;
                }

                env.reachable = true;

                let mut new_envs_to_visit = Vec::<usize>::new();

                for (_, value) in &env.symbols {
                    if let Value::Function { context_function_slot_idx, closure_id } = value {
                        let Function { .. } = self.functions[*context_function_slot_idx];
                        new_envs_to_visit.push(*closure_id);
                    }
                    else if let Value::Instance ( instance_id ) = value {
                        instance_ids_to_mark_stack.push(*instance_id);
                    }
                    else if let Value::Class ( class_id ) = value {
                        let Class { methods, super_class, .. } = &self.classes[*class_id];
                        for value in methods.values() {
                            if let Value::Function { context_function_slot_idx, closure_id } = value {
                                let Function { .. } = self.functions[*context_function_slot_idx];
                                new_envs_to_visit.push(*closure_id);
                            }
                        }

                        let mut s = super_class;

                        while let Some (b) = s {
                            let Value::Class ( class_id ) = b else {
                                panic!("gc: Expected class value");
                            };
                            let Class { methods, super_class, .. } = &self.classes[*class_id];
                            for value in methods.values() {
                                if let Value::Function { context_function_slot_idx, closure_id } = value {
                                    let Function { .. } = self.functions[*context_function_slot_idx];
                                    new_envs_to_visit.push(*closure_id);
                                }
                            }
                            s = super_class;
                        }

                    }
                }

                if let Some(parent_id) = env.parent_key {
                    new_envs_to_visit.push(parent_id);
                }

                env_ids_to_mark_stack.extend(new_envs_to_visit);
            }

            while let Some(instance_id) = instance_ids_to_mark_stack.pop() {

                let Some ( instance ) = self.instances.get_mut(&instance_id) else {
                    continue;
                };

                if instance.reachable {
                    continue;
                }

                instance.reachable = true;

                let mut new_instances_to_visit = Vec::<usize>::new();

                if let Value::Class ( class_id ) = instance.class {

                    let Class { methods, super_class, .. } = &self.classes[class_id];

                    for value in methods.values() {
                        if let Value::Function { context_function_slot_idx, closure_id } = value {
                            let Function { .. } = self.functions[*context_function_slot_idx];
                            env_ids_to_mark_stack.push(*closure_id);
                        }
                    }

                    let mut s = super_class;

                    while let Some (b) = s {
                        let Value::Class ( class_id ) = b else {
                            panic!("gc: Expected class value");
                        };
                        let Class { methods, super_class, .. } = &self.classes[*class_id];
                        for value in methods.values() {
                            if let Value::Function { context_function_slot_idx, closure_id } = value {
                                let Function { .. } = self.functions[*context_function_slot_idx];
                                env_ids_to_mark_stack.push(*closure_id);
                            }
                        }
                        s = super_class;

                    }
                }
                else {
                    panic!("gc: Expected class value");
                }

                for (_, property_value) in &instance.properties {

                    if let Value::Class ( class_id ) = property_value {
                        let Class { methods, super_class, .. } = &self.classes[*class_id];
                        for value in methods.values() {
                            if let Value::Function { context_function_slot_idx, closure_id } = value {
                                let Function { .. } = self.functions[*context_function_slot_idx];
                                env_ids_to_mark_stack.push(*closure_id);
                            }
                        }

                        let mut s = super_class;

                        while let Some (b) = s {
                            let Value::Class ( class_id ) = b else {
                                panic!("gc: Expected class value");
                            };
                            let Class { methods, super_class, .. } = &self.classes[*class_id];
                            for value in methods.values() {
                                if let Value::Function { context_function_slot_idx, closure_id } = value {
                                    let Function { .. } = self.functions[*context_function_slot_idx];
                                    env_ids_to_mark_stack.push(*closure_id);
                                }
                            }
                            s = super_class;
                        }
                    }
                    else if let Value::Function { context_function_slot_idx, closure_id } = property_value {
                        let Function { .. } = self.functions[*context_function_slot_idx];
                        env_ids_to_mark_stack.push(*closure_id);
                    }
                    else if let Value::Instance ( instance_id ) = property_value {
                        new_instances_to_visit.push(*instance_id);
                    }

                }

                instance_ids_to_mark_stack.extend(new_instances_to_visit);

            }

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
