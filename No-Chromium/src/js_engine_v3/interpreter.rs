//! Interpreter - Tree-walking execution
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::ast::*;
use super::value::JsValue;
use super::env::Env;
use super::dom::Dom;
use super::console::Console;
use super::timer::TimerQueue;
use super::fetch::FetchQueue;
use super::builtins::register_builtins;

pub struct Interpreter {
    pub env: Rc<RefCell<Env>>,
    pub global: Rc<RefCell<Env>>,
    pub dom: Rc<RefCell<Dom>>,
    pub console: Rc<RefCell<Console>>,
    pub timers: Rc<RefCell<TimerQueue>>,
    pub fetches: Rc<RefCell<FetchQueue>>,
    pub return_value: Rc<RefCell<Option<JsValue>>>,
    pub had_error: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Env::new()));
        let global = env.clone();
        let dom = Rc::new(RefCell::new(Dom::new()));
        let console = Rc::new(RefCell::new(Console::new()));
        let timers = Rc::new(RefCell::new(TimerQueue::new()));
        let fetches = Rc::new(RefCell::new(FetchQueue::new()));

        // Register built-in objects (Math, JSON, etc.)
        register_builtins(&env, &dom, &console);

        Self {
            env, global, dom, console, timers, fetches,
            return_value: Rc::new(RefCell::new(None)),
            had_error: false,
        }
    }

    pub fn interpret(&mut self, program: &Program) -> Result<JsValue, String> {
        let mut result = JsValue::Undefined;
        for stmt in &program.body {
            self.exec_stmt(stmt)?;
        }
        Ok(result)
    }

    pub fn exec_stmt(&mut self, stmt: &Stmt) -> Result<JsValue, String> {
        match stmt {
            Stmt::VarDecl { name, value, .. } => {
                let v = if let Some(expr) = value { self.eval_expr(expr)? } else { JsValue::Undefined };
                self.env.borrow_mut().set(name.clone(), v);
                Ok(JsValue::Undefined)
            }
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::Return(expr) => {
                let v = if let Some(e) = expr { self.eval_expr(e)? } else { JsValue::Undefined };
                *self.return_value.borrow_mut() = Some(v);
                Ok(JsValue::Undefined)
            }
            Stmt::Block(stmts) => {
                let new_env = Rc::new(RefCell::new(Env::with_parent(self.env.clone())));
                let old_env = self.env.clone();
                self.env = new_env;
                let mut result = Ok(JsValue::Undefined);
                for s in stmts {
                    if let Err(e) = self.exec_stmt(s) { result = Err(e); break; }
                    if self.return_value.borrow().is_some() { break; }
                }
                self.env = old_env;
                result
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond = self.eval_expr(condition)?;
                if cond.is_truthy() {
                    self.exec_block(then_branch)
                } else if let Some(else_stmts) = else_branch {
                    self.exec_block(else_stmts)
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            Stmt::While { condition, body } => {
                loop {
                    let cond = self.eval_expr(condition)?;
                    if !cond.is_truthy() { break; }
                    match self.exec_block(body) {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                    if self.return_value.borrow().is_some() { break; }
                }
                Ok(JsValue::Undefined)
            }
            Stmt::For { init, condition, update, body } => {
                let new_env = Rc::new(RefCell::new(Env::with_parent(self.env.clone())));
                let old_env = self.env.clone();
                self.env = new_env;
                if let Some(init_stmt) = init {
                    self.exec_stmt(init_stmt)?;
                }
                loop {
                    if let Some(cond_expr) = condition {
                        let cond = self.eval_expr(cond_expr)?;
                        if !cond.is_truthy() { break; }
                    }
                    let _ = self.exec_block(body);
                    if self.return_value.borrow().is_some() { break; }
                    if let Some(upd) = update {
                        let _ = self.eval_expr(upd);
                    }
                }
                self.env = old_env;
                Ok(JsValue::Undefined)
            }
            Stmt::Break => Err("BREAK".to_string()),
            Stmt::Continue => Err("CONTINUE".to_string()),
            Stmt::FunctionDecl { name, params, body } => {
                let func = JsValue::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.env.clone(),
                };
                self.env.borrow_mut().set(name.clone(), func);
                Ok(JsValue::Undefined)
            }
            Stmt::Try { body, catch_param, catch_body, finally } => {
                let result = self.exec_block(body);
                if result.is_err() {
                    if let (Some(param), Some(cb)) = (catch_param, catch_body) {
                        let new_env = Rc::new(RefCell::new(Env::with_parent(self.env.clone())));
                        new_env.borrow_mut().set(param.clone(), JsValue::String("Error".to_string()));
                        let old_env = self.env.clone();
                        self.env = new_env;
                        let _ = self.exec_block(cb);
                        self.env = old_env;
                    }
                }
                if let Some(fb) = finally {
                    let _ = self.exec_block(fb);
                }
                Ok(JsValue::Undefined)
            }
            Stmt::Throw(expr) => {
                let v = self.eval_expr(expr)?;
                Err(format!("THROW:{}", v.to_string()))
            }
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<JsValue, String> {
        let mut result = Ok(JsValue::Undefined);
        for s in stmts {
            result = self.exec_stmt(s);
            if result.is_err() { break; }
            if self.return_value.borrow().is_some() { break; }
        }
        result
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<JsValue, String> {
        match expr {
            Expr::Number(n) => Ok(JsValue::Number(*n)),
            Expr::String(s) => Ok(JsValue::String(s.clone())),
            Expr::Bool(b) => Ok(JsValue::Boolean(*b)),
            Expr::Null => Ok(JsValue::Null),
            Expr::Undefined => Ok(JsValue::Undefined),
            Expr::This => {
                self.env.borrow().get("this").ok_or_else(|| "No 'this'".to_string())
            }
            Expr::Identifier(name) => {
                self.env.borrow().get(name)
                    .ok_or_else(|| format!("ReferenceError: {} is not defined", name))
            }
            Expr::Binary { op, left, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary(*op, &l, &r)
            }
            Expr::Unary { op, operand } => {
                let v = self.eval_expr(operand)?;
                self.eval_unary(*op, &v)
            }
            Expr::Call { callee, args } => {
                let func = self.eval_expr(callee)?;
                let evaluated_args: Result<Vec<JsValue>, String> = args.iter().map(|a| self.eval_expr(a)).collect();
                self.call_function(&func, &evaluated_args?)
            }
            Expr::Member { object, property, computed } => {
                let obj = self.eval_expr(object)?;
                self.get_member(&obj, property, *computed)
            }
            Expr::Array(elements) => {
                let mut arr = Vec::new();
                for e in elements {
                    arr.push(self.eval_expr(e)?);
                }
                Ok(JsValue::Array(Rc::new(RefCell::new(arr))))
            }
            Expr::Object(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    map.insert(k.clone(), self.eval_expr(v)?);
                }
                Ok(JsValue::Object(Rc::new(RefCell::new(map))))
            }
            Expr::Assign { target, value } => {
                let v = self.eval_expr(value)?;
                if let Expr::Identifier(name) = &**target {
                    self.env.borrow_mut().set(name.clone(), v.clone());
                    Ok(v)
                } else {
                    Err("Invalid assignment".to_string())
                }
            }
            Expr::New(callee) => {
                let func = self.eval_expr(callee)?;
                self.call_function(&func, &[])
            }
            _ => Ok(JsValue::Undefined),
        }
    }

    fn get_member(&self, obj: &JsValue, property: &str, _computed: bool) -> Result<JsValue, String> {
        match obj {
            JsValue::Object(map) => {
                Ok(map.borrow().get(property).cloned().unwrap_or(JsValue::Undefined))
            }
            JsValue::Array(arr) => {
                if property == "length" {
                    Ok(JsValue::Number(arr.borrow().len() as f64))
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            JsValue::String(s) => {
                if property == "length" {
                    Ok(JsValue::Number(s.len() as f64))
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            JsValue::DomElement(node) => {
                // Use DOM API to get property
                Ok(self.dom.borrow().get_property(&node.borrow().tag_name, property))
            }
            _ => Ok(JsValue::Undefined),
        }
    }

    fn eval_binary(&self, op: BinaryOp, l: &JsValue, r: &JsValue) -> Result<JsValue, String> {
        match (l, r) {
            (JsValue::Number(a), JsValue::Number(b)) => {
                let result = match op {
                    BinaryOp::Add => a + b, BinaryOp::Sub => a - b, BinaryOp::Mul => a * b,
                    BinaryOp::Div => a / b, BinaryOp::Mod => a % b,
                    BinaryOp::Lt => return Ok(JsValue::Boolean(a < b)),
                    BinaryOp::Gt => return Ok(JsValue::Boolean(a > b)),
                    BinaryOp::LtEq => return Ok(JsValue::Boolean(a <= b)),
                    BinaryOp::GtEq => return Ok(JsValue::Boolean(a >= b)),
                    BinaryOp::Eq | BinaryOp::StrictEq => return Ok(JsValue::Boolean(a == b)),
                    BinaryOp::Ne | BinaryOp::StrictNe => return Ok(JsValue::Boolean(a != b)),
                    BinaryOp::And => return Ok(JsValue::Boolean(*a != 0.0 && *b != 0.0)),
                    BinaryOp::Or => return Ok(JsValue::Boolean(*a != 0.0 || *b != 0.0)),
                    _ => return Ok(JsValue::Boolean(false)),
                };
                Ok(JsValue::Number(result))
            }
            (JsValue::String(a), JsValue::String(b)) => {
                match op {
                    BinaryOp::Add => Ok(JsValue::String(format!("{}{}", a, b))),
                    BinaryOp::Eq | BinaryOp::StrictEq => Ok(JsValue::Boolean(a == b)),
                    BinaryOp::Ne | BinaryOp::StrictNe => Ok(JsValue::Boolean(a != b)),
                    _ => Ok(JsValue::Boolean(false)),
                }
            }
            (JsValue::String(a), JsValue::Number(b)) => {
                if matches!(op, BinaryOp::Add) {
                    Ok(JsValue::String(format!("{}{}", a, b.to_string())))
                } else {
                    Ok(JsValue::Boolean(false))
                }
            }
            (JsValue::Number(a), JsValue::String(b)) => {
                if matches!(op, BinaryOp::Add) {
                    Ok(JsValue::String(format!("{}{}", a.to_string(), b)))
                } else {
                    Ok(JsValue::Boolean(false))
                }
            }
            (JsValue::Boolean(a), JsValue::Boolean(b)) => {
                match op {
                    BinaryOp::Eq | BinaryOp::StrictEq => Ok(JsValue::Boolean(a == b)),
                    BinaryOp::Ne | BinaryOp::StrictNe => Ok(JsValue::Boolean(a != b)),
                    BinaryOp::And => Ok(JsValue::Boolean(*a && *b)),
                    BinaryOp::Or => Ok(JsValue::Boolean(*a || *b)),
                    _ => Ok(JsValue::Boolean(false)),
                }
            }
            _ => Ok(JsValue::Boolean(false)),
        }
    }

    fn eval_unary(&self, op: UnaryOp, v: &JsValue) -> Result<JsValue, String> {
        match op {
            UnaryOp::Neg => Ok(JsValue::Number(-v.to_number())),
            UnaryOp::Not => Ok(JsValue::Boolean(!v.is_truthy())),
            UnaryOp::BitNot => Ok(JsValue::Number(!(v.to_number() as i64) as f64)),
            UnaryOp::TypeOf => Ok(JsValue::String(v.type_of().to_string())),
            UnaryOp::Void => Ok(JsValue::Undefined),
            UnaryOp::Delete => Ok(JsValue::Boolean(true)),
        }
    }

    fn call_function(&mut self, func: &JsValue, args: &[JsValue]) -> Result<JsValue, String> {
        match func {
            JsValue::Function { name, params, body, closure } => {
                let new_env = Rc::new(RefCell::new(Env::with_parent(closure.clone())));
                for (param, arg) in params.iter().zip(args.iter()) {
                    new_env.borrow_mut().set(param.clone(), arg.clone());
                }
                let old_env = self.env.clone();
                self.env = new_env;
                *self.return_value.borrow_mut() = None;
                for s in body {
                    let _ = self.exec_stmt(s);
                    if self.return_value.borrow().is_some() { break; }
                }
                self.env = old_env;
                let result = self.return_value.borrow_mut().take().unwrap_or(JsValue::Undefined);
                Ok(result)
            }
            JsValue::NativeFunction { name, func } => {
                func(args)
            }
            _ => Err(format!("{} is not a function", func.type_of())),
        }
    }
}
