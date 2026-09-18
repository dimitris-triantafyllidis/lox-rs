use crate::lexer::*;
use crate::parser::*;
use crate::context::*;
use crate::interpreter::*;

pub struct SemanticPass {
    pub context: Context
}

impl SemanticPass {

    pub fn new() -> Self {

        let mut context = Context::new();
        context.push_new_environment_auto();

        Self {
            context
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

    pub fn visit_function_call(self: &mut Self, expr: &mut Expression) {

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
            Expression::Assignment      {..} => self.visit_assignment(expr),
            Expression::LogicalOr       {..} => self.visit_logical_or(expr),
            Expression::LogicalAnd      {..} => self.visit_logical_and(expr),
            Expression::Call            {..} => self.visit_function_call(expr),
            _ => panic!()
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
            Statement::FunctionDeclaration {..} => self.visit_function_declaration_statement(statement),
            Statement::ClassDeclaration    {..} => self.visit_class_declaration_statement(statement),
            Statement::Block               {..} => self.visit_block_statement(statement),
            Statement::If                  {..} => self.visit_if_statement(statement),
            _ => panic!()
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
            self.visit_expression(expr);
        }
        else if let Statement::Return { expr: None } = statement { }
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

            self.context.insert_symbol (
                &id.lexeme,
                Value::Function (
                    pars.clone(),
                    *body.clone(),
                    self.context.get_current_environment_id()
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

        if let Statement::ClassDeclaration { id, .. } = statement {

            self.context.insert_symbol (
                &id.lexeme,
                Value::Nil
            );
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
