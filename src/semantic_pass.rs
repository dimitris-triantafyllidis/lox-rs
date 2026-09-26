use crate::lexer::*;
use crate::parser::*;
use crate::context::*;
use crate::interpreter::*;

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

pub struct SemanticPass {
    pub context: Context,
    pub function_context_stack: Vec<FunctionContext>,
    pub class_context_stack: Vec<ClassContext>
}

impl SemanticPass {

    pub fn new() -> Self {

        let mut context = Context::new();
        context.push_new_environment_auto();

        let function_context_stack = Vec::<FunctionContext>::new();
        let class_context_stack = Vec::<ClassContext>::new();

        Self {
            context,
            function_context_stack,
            class_context_stack
        }
    }

    pub fn visit_literal(self: &mut Self, expr: &mut Expression) {

        match expr {
            Expression::Literal {..} => { },
            _ => panic!()
        }
    }

    pub fn visit_variable(self: &mut Self, expr: &mut Expression) {

        match expr {
            Expression::Variable {
                identifier: id,
                lookup_hop_count: hop_count
            } => {
                if !hop_count.is_none() {
                    panic!();
                }
                self.context.get_symbol_value(&id.lexeme, hop_count);
            },
            _ => {
                panic!()
            }
        }
    }

    pub fn visit_this(self: &Self, expr: &mut Expression) {

        match expr {
            Expression::This => {
                if self.class_context_stack.is_empty() {
                    panic!("Unexpected 'this' outside a class context");
                }
                else {
                    self.context.get_symbol_value(&"this".to_string(), &mut None);
                }
            },
            _ => {
                panic!()
            }
        }
    }

    pub fn visit_super(self: &Self, expr: &mut Expression) {

        match expr {
            Expression::Super {
                lookup_hop_count: hop_count,
                ..
            } => {
                if !hop_count.is_none() {
                    panic!();
                }
                self.context.get_symbol_value(&"super".to_string(), hop_count);
            },
            _ => {
                panic!()
            }
        }
    }

    pub fn visit_unary(self: &mut Self, expr: &mut Expression) {

        if let Expression::UnaryOperation { operator: _, right: rhs } = expr {
            self.visit_expression(rhs);
        }
        else {
            panic!()
        }
    }

    pub fn visit_parentheses(self: &mut Self, expr: &mut Expression) {

        if let Expression::Parentheses { expression: e } = expr {
            self.visit_expression(e);
        }
        else {
            panic!()
        }

    }

    pub fn visit_get(self: &mut Self, expr: &mut Expression) {

        match expr {
            Expression::Get { instance, .. } => {
                self.visit_expression(instance);
            },
            _ => {
                panic!()
            }
        }
    }

    pub fn visit_set(self: &mut Self, expr: &mut Expression) {

        match expr {
            Expression::Set { instance, value, .. } => {
                self.visit_expression(instance);
                self.visit_expression(value);
            },
            _ => {
                panic!()
            }
        }
    }

    pub fn visit_assignment(self: &mut Self, expr: &mut Expression) {

        match expr {
            Expression::Assignment {
                left: lhs,
                expression: rhs,
                lookup_hop_count: hop_count
            } => {
                self.visit_expression(rhs);
                self.context.set_symbol_value(&lhs.lexeme, Value::Nil, hop_count);
            },
            _ => {
                panic!()
            }
        }

    }

    pub fn visit_binary(self: &mut Self, expr: &mut Expression) {

        if let Expression::BinaryOperation { operator: op, left: lhs, right: rhs } = expr {

            self.visit_expression(lhs);
            self.visit_expression(rhs);

            match op.kind {
                TokenKind::Plus         => { },
                TokenKind::Minus        => { },
                TokenKind::Star         => { },
                TokenKind::Slash        => { },
                TokenKind::EqualEqual   => { },
                TokenKind::BangEqual    => { },
                TokenKind::Less         => { },
                TokenKind::LessEqual    => { },
                TokenKind::Greater      => { },
                TokenKind::GreaterEqual => { },
                _ => panic!()
            }
        }
        else {
            panic!()
        }

    }

    pub fn visit_logical_or(self: &mut Self, expr: &mut Expression) {

        if let Expression::LogicalOr { left: lhs, right: rhs } = expr {

            self.visit_expression(lhs);
            self.visit_expression(rhs);
        }
        else {
            panic!()
        }

    }

    pub fn visit_logical_and(self: &mut Self, expr: &mut Expression) {

        if let Expression::LogicalAnd { left: lhs, right: rhs } = expr {

            self.visit_expression(lhs);
            self.visit_expression(rhs);
        }
        else {
            panic!()
        }

    }

    pub fn visit_call(self: &mut Self, expr: &mut Expression) {

        if let Expression::Call { callee, arguments } = expr {
            self.visit_expression(callee);
            for argument in arguments {
                self.visit_expression(argument);
            }
        }
    }

    pub fn visit_expression(self: &mut Self, expr: &mut Expression) {

        match expr {
            Expression::Literal         {..} => self.visit_literal(expr),
            Expression::UnaryOperation  {..} => self.visit_unary(expr),
            Expression::BinaryOperation {..} => self.visit_binary(expr),
            Expression::Parentheses     {..} => self.visit_parentheses(expr),
            Expression::Variable        {..} => self.visit_variable(expr),
            Expression::This            {..} => self.visit_this(expr),
            Expression::Assignment      {..} => self.visit_assignment(expr),
            Expression::LogicalOr       {..} => self.visit_logical_or(expr),
            Expression::LogicalAnd      {..} => self.visit_logical_and(expr),
            Expression::Call            {..} => self.visit_call(expr),
            Expression::Get             {..} => self.visit_get(expr),
            Expression::Set             {..} => self.visit_set(expr),
            Expression::Super           {..} => self.visit_super(expr),
        }

    }

    pub fn visit_statement(self: &mut Self, statement: &mut Statement) {

        match statement {
            Statement::Expression          {..} => self.visit_expression_statement(statement),
            Statement::Print               {..} => self.visit_print_statement(statement),
            Statement::Return              {..} => self.visit_return_statement(statement),
            Statement::While               {..} => self.visit_while_statement(statement),
            Statement::For                 {..} => self.visit_for_statement(statement),
            Statement::VariableDeclaration {..} => self.visit_variable_declaration_statement(statement),
            Statement::FunctionDeclaration {..} => {
                self.function_context_stack.push(FunctionContext::Function);
                self.visit_function_declaration_statement(statement);
                self.function_context_stack.pop();
            },
            Statement::ClassDeclaration    {..} => self.visit_class_declaration_statement(statement),
            Statement::Block               {..} => self.visit_block_statement(statement),
            Statement::If                  {..} => self.visit_if_statement(statement),
        }

    }

    pub fn visit_expression_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::Expression { expr } = statement {
            self.visit_expression(expr);
        }
        else {
            panic!();
        }

    }

    pub fn visit_print_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::Print { expr } = statement {
            self.visit_expression(expr);
        }
        else {
            panic!();
        }

    }

    pub fn visit_return_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::Return { expr: Some(expr) } = statement {
            if self.function_context_stack.is_empty() {
                panic!("Unexpected 'return' outside a function context")
            }
            else {
                self.visit_expression(expr);
            }
        }
        else if let Statement::Return { expr: None } = statement {
            if self.function_context_stack.is_empty() {
                panic!("Unexpected 'return' outside a function context")
            }
        }
        else {
            panic!();
        }

    }

    pub fn visit_while_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::While { condition, body } = statement {
            self.visit_expression(condition);
            self.visit_statement(body);
        }
        else {
            panic!();
        }
    }

    pub fn visit_for_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::For {
            init: initializer_statement,
            cond: condition_expression,
            incr: increment_expression,
            body: body_statement,
        } = statement {

            self.context.push_new_environment_auto();

            if let Some(statement) = initializer_statement {
                self.visit_statement(statement);
            }

            if let Some(ce) = condition_expression {
                self.visit_expression(ce);
            }

            self.visit_statement(body_statement);

            if let Some(ie) = increment_expression {
                self.visit_expression(ie);
            }

            self.context.pop_environment();
        }
        else {
            panic!();
        }

    }

    pub fn visit_if_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::If { condition, then_statement, else_statement } = statement {
            self.visit_expression(condition);
            self.visit_statement(then_statement);
            if let Some(statement) = else_statement {
                self.visit_statement(statement);
            }
        }
        else {
            panic!();
        }

    }

    pub fn visit_function_declaration_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::FunctionDeclaration { id, pars, body } = statement {

            let function_context = self.function_context_stack.last().cloned().unwrap();
            let mut closure_id = self.context.get_current_environment_id();

            if function_context == FunctionContext::Method || function_context == FunctionContext::Initializer {
                self.context.push_new_environment(Some(closure_id));
                self.context.insert_symbol(&"this".to_string(), Value::Nil);
                closure_id = self.context.next_id - 1;
            }

            self.context.insert_symbol (
                &id.lexeme,
                Value::Function (
                    Function {
                        pars: pars.clone(),
                        body: *body.clone(),
                        closure_id
                    }
                )
            );

            self.context.push_new_environment_auto();

            for i in 0..pars.len() {
                self.context.insert_symbol (
                    &pars[i].lexeme,
                    Value::Nil
                );
            }

            if let Statement::Block { statements } = body.as_mut() {
                for ref mut statement in statements {
                    self.visit_statement(statement);
                }

                if function_context == FunctionContext::Method || function_context == FunctionContext::Initializer {
                    self.context.pop_environment();
                }

            }
            else {
                panic!("Expected block statement");
            }

            self.context.pop_environment();
        }
        else {
            panic!();
        }

    }

    pub fn visit_class_declaration_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::ClassDeclaration { id, method_decls, super_class } = statement {

            if let Some(expr) = super_class {
                self.visit_variable(expr);
            }

            self.class_context_stack.push(ClassContext::Class);

            self.context.insert_symbol (
                &id.lexeme,
                Value::Nil
            );

            if let Some(..) = super_class {
                self.context.push_new_environment_auto();
                self.context.insert_symbol(&"super".to_string(), Value::Nil);
            }

            for method_decl in method_decls {
                if let Statement::FunctionDeclaration { id, .. } = method_decl {
                    if id.lexeme == "init" {
                        self.function_context_stack.push(FunctionContext::Initializer);
                    }
                    else {
                        self.function_context_stack.push(FunctionContext::Method);
                    }
                    self.visit_function_declaration_statement(method_decl);
                    self.function_context_stack.pop();
                }
                else {
                    panic!("Expected function declaration statement");
                }
            }

            if let Some(..) = super_class {
                self.context.pop_environment();
            }

            self.class_context_stack.pop();
        }
        else {
            panic!();
        }

    }

    pub fn visit_variable_declaration_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::VariableDeclaration { id, init } = statement {

            if let Some(e) = init {
                self.visit_expression(e);
            }

            self.context.insert_symbol(&id.lexeme, Value::Nil);
        }
        else {
            panic!();
        }

    }

    pub fn visit_block_statement(self: &mut Self, statement: &mut Statement) {

        if let Statement::Block { statements } = statement {
            self.context.push_new_environment_auto();
            for statement in statements {
                self.visit_statement(statement);
            }
            self.context.pop_environment();
        }
        else {
            panic!();
        }
    }

    pub fn visit(self: &mut Self, statements: &mut Vec<Statement>) {
        for statement in statements {
            self.visit_statement(statement);
        }
        return;
    }

}
