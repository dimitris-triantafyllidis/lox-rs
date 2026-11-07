use std::collections::HashMap;

use crate::lexer as lexer;
use crate::context as ctx;
use crate::interpreter as ipr;

use crate::ast as ast;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionContext {
    Function,
    Method,
    Initializer
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassContext {
    Class
}

pub struct SemanticPass<'a> {
    pub context: ctx::Context<'a>,
    pub function_context_stack: Vec<FunctionContext>,
    pub class_context_stack: Vec<ClassContext>,
    pub defs: ctx::Definitions<'a>
}

impl<'a> SemanticPass<'a> {

    pub fn new() -> Self {

        let mut defs = ctx::Definitions::new();
        let mut context = ctx::Context::new();
        context.push_new_environment_auto();

        let function_id =
            defs.insert_function (
                ipr::FunctionDefinition {
                    pars: Vec::new(),
                    body: ast::stmt::Statement::Block (
                        ast::stmt::Block {
                            statements: Vec::new()
                        }
                    ),
                    is_native: true,
                    native_function: ipr::NativeFunction::Clock,
                    is_initializer: false,
                    is_method: false,
                    name: "clock".to_string()
                }
            );

        context.insert_symbol (
            "class",
            ipr::Value::Nil
        );

        let function_context_stack = Vec::<FunctionContext>::new();
        let class_context_stack = Vec::<ClassContext>::new();

        if function_id != 0 {
            panic!("Internal resolver error");
        }

        Self {
            context,
            defs,
            function_context_stack,
            class_context_stack,
        }
    }

    pub fn compile_error(&self, message: &str) -> ! {
        eprintln!("Compile error:, '{}'", message);
        std::process::exit(1);
    }

    pub fn visit_literal(&mut self, _: &mut ast::expr::Literal<'a>) { }

    pub fn visit_variable(&mut self, expr: &mut ast::expr::Variable<'a>) {

        let ast::expr::Variable { identifier: id, lookup_hop_count, env_slot_idx } = expr;
        if !lookup_hop_count.is_none() {
            panic!();
        }
        self.context.find_symbol(id.lexeme, lookup_hop_count, env_slot_idx);
    }

    pub fn visit_this(&self) {

        if self.class_context_stack.is_empty() {
            self.compile_error("Can't use 'this' outside of a class.'");
        }
        else {
            self.context.find_symbol("this", &mut None, &mut None);
        }
    }

    pub fn visit_super(&self, expr: &mut ast::expr::Super<'a>) {

        if self.class_context_stack.is_empty() {
            self.compile_error("Can't use 'super' outside of a class.'");
        }
        let ast::expr::Super { lookup_hop_count, .. } = expr;
        if !lookup_hop_count.is_none() {
            panic!();
        }
        self.context.find_symbol("super", &mut None, &mut None);
    }

    pub fn visit_unary(&mut self, expr: &mut ast::expr::UnaryOperation<'a>) {

        let ast::expr::UnaryOperation { operator: _, right: rhs } = expr;
        self.visit_expression(rhs);
    }

    pub fn visit_parentheses(&mut self, expr: &mut ast::expr::Parentheses<'a>) {

        let ast::expr::Parentheses { expression: e } = expr;
        self.visit_expression(e);
    }

    pub fn visit_get(&mut self, expr: &mut ast::expr::Get<'a>) {

        let ast::expr::Get { instance, .. } = expr;
        self.visit_expression(instance);
    }

    pub fn visit_set(&mut self, expr: &mut ast::expr::Set<'a>) {

        let ast::expr::Set { instance, value, .. } = expr;
        self.visit_expression(instance);
        self.visit_expression(value);
    }

    pub fn visit_assignment(&mut self, expr: &mut ast::expr::Assignment<'a>) {

        let ast::expr::Assignment { left: lhs, expression: rhs, lookup_hop_count, env_slot_idx } = expr;
        self.visit_expression(rhs);
        self.context.find_symbol(lhs.lexeme, lookup_hop_count, env_slot_idx);
    }

    pub fn visit_binary(&mut self, expr: &mut ast::expr::BinaryOperation<'a>) {

        let ast::expr::BinaryOperation { operator: op, left: lhs, right: rhs } = expr;

        self.visit_expression(lhs);
        self.visit_expression(rhs);

        match op.kind {
            lexer::TokenKind::Plus         => { },
            lexer::TokenKind::Minus        => { },
            lexer::TokenKind::Star         => { },
            lexer::TokenKind::Slash        => { },
            lexer::TokenKind::EqualEqual   => { },
            lexer::TokenKind::BangEqual    => { },
            lexer::TokenKind::Less         => { },
            lexer::TokenKind::LessEqual    => { },
            lexer::TokenKind::Greater      => { },
            lexer::TokenKind::GreaterEqual => { },
            _ => panic!()
        }

    }

    pub fn visit_logical_or(&mut self, expr: &mut ast::expr::LogicalOr<'a>) {

        let ast::expr::LogicalOr { left: lhs, right: rhs } = expr;
        self.visit_expression(lhs);
        self.visit_expression(rhs);
    }

    pub fn visit_logical_and(&mut self, expr: &mut ast::expr::LogicalAnd<'a>) {

        let ast::expr::LogicalAnd { left: lhs, right: rhs } = expr;
        self.visit_expression(lhs);
        self.visit_expression(rhs);
    }

    pub fn visit_call(&mut self, expr: &mut ast::expr::Call<'a>) {

        let ast::expr::Call { callee, arguments } = expr;
        self.visit_expression(callee);
        for argument in arguments {
            self.visit_expression(argument);
        }
    }

    pub fn visit_expression(&mut self, expr: &mut ast::expr::Expression<'a>) {

        match expr {
            ast::expr::Expression::Literal         (expr) => self.visit_literal(expr),
            ast::expr::Expression::UnaryOperation  (expr) => self.visit_unary(expr),
            ast::expr::Expression::BinaryOperation (expr) => self.visit_binary(expr),
            ast::expr::Expression::Parentheses     (expr) => self.visit_parentheses(expr),
            ast::expr::Expression::Variable        (expr) => self.visit_variable(expr),
            ast::expr::Expression::This                   => self.visit_this(),
            ast::expr::Expression::Assignment      (expr) => self.visit_assignment(expr),
            ast::expr::Expression::LogicalOr       (expr) => self.visit_logical_or(expr),
            ast::expr::Expression::LogicalAnd      (expr) => self.visit_logical_and(expr),
            ast::expr::Expression::Call            (expr) => self.visit_call(expr),
            ast::expr::Expression::Get             (expr) => self.visit_get(expr),
            ast::expr::Expression::Set             (expr) => self.visit_set(expr),
            ast::expr::Expression::Super           (expr) => self.visit_super(expr),
        }

    }

    pub fn visit_statement(&mut self, statement: &mut ast::stmt::Statement<'a>) {

        match statement {
            ast::stmt::Statement::Expression(stmt)          => self.visit_expression_statement(stmt),
            ast::stmt::Statement::Print(stmt)               => self.visit_print_statement(stmt),
            ast::stmt::Statement::Return(stmt)              => self.visit_return_statement(stmt),
            ast::stmt::Statement::While(stmt)               => self.visit_while_statement(stmt),
            ast::stmt::Statement::For(stmt)                 => self.visit_for_statement(stmt),
            ast::stmt::Statement::VariableDeclaration(stmt) => self.visit_variable_declaration_statement(stmt),
            ast::stmt::Statement::FunctionDeclaration(stmt) => {
                self.function_context_stack.push(FunctionContext::Function);
                self.visit_function_declaration_statement(stmt);
                self.function_context_stack.pop();
            },
            ast::stmt::Statement::ClassDeclaration(stmt)   => self.visit_class_declaration_statement(stmt),
            ast::stmt::Statement::Block(stmt)              => self.visit_block_statement(stmt),
            ast::stmt::Statement::If(stmt)                 => self.visit_if_statement(stmt),
        }

    }

    pub fn visit_expression_statement(&mut self, stmt: &mut ast::stmt::Expression<'a>) {
        let ast::stmt::Expression { expr } = stmt;
        self.visit_expression(expr);
    }

    pub fn visit_print_statement(&mut self, stmt: &mut ast::stmt::Print<'a>) {
        let ast::stmt::Print { expr } = stmt;
        self.visit_expression(expr);
    }

    pub fn visit_return_statement(&mut self, stmt: &mut ast::stmt::Return<'a>) {

        if self.function_context_stack.is_empty() {
            panic!("Can't return from top-level code.")
        }

        let ast::stmt::Return { expr } = stmt;

        match expr {
            Some ( ast::stmt::Expression { expr } ) => {
                if self.function_context_stack.last().unwrap() == &FunctionContext::Initializer {
                    panic!("Error at 'return': Can't return a value from an initializer.");
                }
                self.visit_expression(expr);
            },
            None => { }
        }

    }

    pub fn visit_while_statement(&mut self, stmt: &mut ast::stmt::While<'a>) {
        let ast::stmt::While { condition: ast::stmt::Expression { expr }, body } = stmt;
        self.visit_expression(expr);
        self.visit_statement(body);
    }

    pub fn visit_for_statement(&mut self, stmt: &mut ast::stmt::For<'a>) {

        let ast::stmt::For {
            init: initializer_statement,
            cond: condition_expression,
            incr: increment_expression,
            body: body_statement,
        } = stmt;

        self.context.push_new_environment_auto();

        if let Some ( statement ) = initializer_statement {
            self.visit_statement(statement);
        }

        if let Some ( ast::stmt::Expression { expr } ) = condition_expression {
            self.visit_expression(expr);
        }

        self.visit_statement(body_statement);

        if let Some ( ast::stmt::Expression { expr } ) = increment_expression {
            self.visit_expression(expr);
        }

        self.context.pop_environment();
    }

    pub fn visit_if_statement(&mut self, stmt: &mut ast::stmt::If<'a>) {

        let ast::stmt::If { condition: ast::stmt::Expression { expr }, then_statement, else_statement } = stmt;
        self.visit_expression(expr);
        self.visit_statement(then_statement);
        if let Some(statement) = else_statement {
            self.visit_statement(statement);
        }
    }

    pub fn visit_function_declaration_statement(&mut self, stmt: &mut ast::stmt::FunctionDeclaration<'a>) {

        let ast::stmt::FunctionDeclaration { id, pars, body, context_function_slot_idx, .. } = stmt;

        if !context_function_slot_idx.is_none() {
            panic!("Internal resolver error");
        }
        else {
            *context_function_slot_idx = Some(self.defs.functions.len());
        }

        let function_context = self.function_context_stack.last().cloned().unwrap();
        let mut closure_id = self.context.get_current_environment_id();

        if function_context == FunctionContext::Method || function_context == FunctionContext::Initializer {
            self.context.push_new_environment(Some(closure_id));
            self.context.insert_symbol("this", ipr::Value::Nil);
            closure_id = self.context.next_env_id - 1;
        }

        let mut par_lexemes = Vec::<&'a str>::new();

        for par in pars.clone() {
            par_lexemes.push(par.lexeme);
        }

        let is_initializer = function_context == FunctionContext::Initializer;
        let is_method      = function_context == FunctionContext::Method;

        let function_id = self.defs.functions.len();

        self.defs.insert_function (
            ipr::FunctionDefinition {
                pars: par_lexemes,
                body: *body.clone(),
                is_native: false,
                native_function: ipr::NativeFunction::None,
                is_initializer,
                is_method,
                name: id.lexeme.to_string()
            }
        );

        self.context.insert_symbol (
            &id.lexeme,
            ipr::Value::Function (
                ipr::Function {
                    context_function_slot_idx: context_function_slot_idx.unwrap(),
                    closure_id
                }
            )
        );

        self.context.push_new_environment_auto();

        for i in 0..pars.len() {
            self.context.insert_symbol (
                pars[i].lexeme,
                ipr::Value::Nil
            );
        }

        let ast::stmt::Statement::Block ( ast::stmt::Block { statements } ) = body.as_mut() else {
            panic!("Expected block statement");
        };

        for statement in statements {
            self.visit_statement(statement);
        }

        if function_context == FunctionContext::Method || function_context == FunctionContext::Initializer {
            self.context.pop_environment();
        }

        self.defs.functions[function_id].body = *body.clone();

        self.context.pop_environment();
    }

    pub fn visit_class_declaration_statement(&mut self, stmt: &mut ast::stmt::ClassDeclaration<'a>) {

        let ast::stmt::ClassDeclaration { id, method_decls, super_class, context_class_slot_idx } = stmt;

        let super_class_value: Option<Box<ipr::Value>>;
        let super_class_context_class_slot_idx: Option<usize>;

        if let Some ( expr ) = super_class {

            let ast::expr::Expression::Variable(expr) = expr else {
                panic!("Expected variable expression");
            };

            self.visit_variable(expr);
            super_class_value = Some (
                Box::new (
                    self.context.get_symbol_value (
                        expr.identifier.lexeme,
                        &mut expr.lookup_hop_count,
                        &mut expr.env_slot_idx
                    )
                )
            );
            let ipr::Value::Class ( ipr::Class { context_class_slot_idx, .. } ) = super_class_value.as_ref().unwrap().as_ref().clone() else {
                panic!("Superclass must be a class.");
            };
            super_class_context_class_slot_idx = Some(context_class_slot_idx);
        }
        else {
            super_class_value = None;
            super_class_context_class_slot_idx = None;
        }

        self.class_context_stack.push(ClassContext::Class);

        self.context.insert_symbol (
            &id.lexeme,
            ipr::Value::Nil
        );

        if let Some(..) = super_class {
            self.context.push_new_environment_auto();
            self.context.insert_symbol("super", ipr::Value::Nil);
        }

        for method_decl in method_decls.iter_mut() {
            let ast::stmt::FunctionDeclaration { id, .. } = method_decl;
            if id.lexeme == "init" {
                self.function_context_stack.push(FunctionContext::Initializer);
            }
            else {
                self.function_context_stack.push(FunctionContext::Method);
            }
            self.visit_function_declaration_statement(method_decl);
            self.function_context_stack.pop();
        }

        let mut class_method_map = HashMap::<lexer::Token, usize>::new();

        for method_decl in method_decls {
            let ast::stmt::FunctionDeclaration { id, context_function_slot_idx, .. } = method_decl;

            class_method_map.insert (
                id.clone(),
                context_function_slot_idx.unwrap()
            );
        }

        let mut class_method_lexeme_map = HashMap::<&'a str, usize>::new();

        for (method_token, value) in class_method_map {
            class_method_lexeme_map.insert(method_token.lexeme, value);
        }

        let class_closure_id = self.context.get_current_environment_id();

        if let Some(..) = super_class {
            self.context.pop_environment();
        }

        if let Some(..) = context_class_slot_idx {
            panic!("Internal resolver error");
        }
        else {
            *context_class_slot_idx = Some(self.defs.classes.len());
        }

        self.defs.insert_class (
            ipr::ClassDefinition {
                methods: class_method_lexeme_map,
                super_class: super_class_context_class_slot_idx,
                name: id.lexeme.to_string()
            }
        );

        self.context.set_symbol_value (
            &id.lexeme,
            ipr::Value::Class (
                ipr::Class {
                    context_class_slot_idx: context_class_slot_idx.unwrap(),
                    closure_id: class_closure_id,
                    super_class: super_class_value
                },
            ),
            &mut Some(0),
            &mut None
        );

        self.class_context_stack.pop();

    }

    pub fn visit_variable_declaration_statement(&mut self, stmt: &mut ast::stmt::VariableDeclaration<'a>) {

        let ast::stmt::VariableDeclaration { id, init } = stmt;

        if let Some ( ast::stmt::Expression { expr } ) = init {
            self.visit_expression(expr);
        }

        self.context.insert_symbol(id.lexeme, ipr::Value::Nil);
    }

    pub fn visit_block_statement(&mut self, stmt: &mut ast::stmt::Block<'a>) {

        let ast::stmt::Block { statements } = stmt;
        self.context.push_new_environment_auto();
        for statement in statements {
            self.visit_statement(statement);
        }
        self.context.pop_environment();
    }

    pub fn visit(&mut self, statements: &mut Vec<ast::stmt::Statement<'a>>) {
        for statement in statements {
            self.visit_statement(statement);
        }
        return;
    }

}
