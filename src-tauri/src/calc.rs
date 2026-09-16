//! 计算稿纸插件后端
//!
//! 无依赖的递归下降表达式求值器，支持：
//! - 运算符 + - * / %（模） ^ 或 **（幂，右结合）、一元负号、括号
//! - 后缀百分比：数字后紧跟 % 表示 /100（如 50% * 200 = 100；带空格的 7 % 3 为取模）
//! - 函数：sin cos tan asin acos atan sqrt cbrt ln log log2 abs round floor ceil exp min max pow
//!   （三角函数使用弧度）
//! - 常量：pi e tau；稿纸内可变量赋值（a = 1+2）并引用，ans 为上一个成功结果
//!
//! 对外命令：evaluate_expression（单表达式）、evaluate_scratchpad（整页稿纸逐行求值）、
//! load/save_calc_lines（持久化到 calc.json，上限 200 行）

use serde::Serialize;
use std::collections::HashMap;
use tauri::{command, AppHandle};
use tauri_plugin_store::StoreExt;

/// 稿纸最多保留的行数
const MAX_CALC_LINES: usize = 200;

// ===== 词法 =====

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Op(char), // + - * / % ^ ( ) ,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // 数字（含小数点，支持 .5 写法）
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            let mut seen_dot = false;
            while i < chars.len() {
                let ch = chars[i];
                if ch.is_ascii_digit() {
                    i += 1;
                } else if ch == '.' && !seen_dot {
                    seen_dot = true;
                    i += 1;
                } else {
                    break;
                }
            }
            let text: String = chars[start..i].iter().collect();
            let value: f64 = text
                .parse()
                .map_err(|_| format!("无效的数字: {}", text))?;
            // 后缀百分比：数字后紧跟 % 且无空格
            let mut value = value;
            if i < chars.len() && chars[i] == '%' {
                value /= 100.0;
                i += 1;
            }
            tokens.push(Token::Number(value));
            continue;
        }

        // 标识符（变量 / 函数 / 常量）
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token::Ident(chars[start..i].iter().collect()));
            continue;
        }

        // ** 作为幂
        if c == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            tokens.push(Token::Op('^'));
            i += 2;
            continue;
        }

        match c {
            '+' | '-' | '*' | '/' | '%' | '^' | '(' | ')' | ',' => {
                tokens.push(Token::Op(c));
                i += 1;
            }
            _ => return Err(format!("无法识别的字符: {}", c)),
        }
    }

    Ok(tokens)
}

// ===== 语法 / 求值 =====

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    vars: &'a HashMap<String, f64>,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn eat_op(&mut self, op: char) -> bool {
        if matches!(self.peek(), Some(Token::Op(o)) if *o == op) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_op(&mut self, op: char) -> Result<(), String> {
        if self.eat_op(op) {
            Ok(())
        } else {
            Err(format!("缺少 '{}'，表达式不完整", op))
        }
    }

    // expr := term (('+'|'-') term)*
    fn parse_expr(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;
        loop {
            if self.eat_op('+') {
                left += self.parse_term()?;
            } else if self.eat_op('-') {
                left -= self.parse_term()?;
            } else {
                return Ok(left);
            }
        }
    }

    // term := unary (('*'|'/'|'%') unary)*
    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_unary()?;
        loop {
            if self.eat_op('*') {
                left *= self.parse_unary()?;
            } else if self.eat_op('/') {
                let rhs = self.parse_unary()?;
                if rhs == 0.0 {
                    return Err("除数不能为 0".to_string());
                }
                left /= rhs;
            } else if self.eat_op('%') {
                let rhs = self.parse_unary()?;
                if rhs == 0.0 {
                    return Err("模数不能为 0".to_string());
                }
                left %= rhs;
            } else {
                return Ok(left);
            }
        }
    }

    // unary := ('-'|'+')* power
    fn parse_unary(&mut self) -> Result<f64, String> {
        if self.eat_op('-') {
            return Ok(-self.parse_unary()?);
        }
        if self.eat_op('+') {
            return self.parse_unary();
        }
        self.parse_power()
    }

    // power := primary ('^' unary)?   右结合
    fn parse_power(&mut self) -> Result<f64, String> {
        let base = self.parse_primary()?;
        if self.eat_op('^') {
            let exp = self.parse_unary()?;
            let result = base.powf(exp);
            if !result.is_finite() {
                return Err("结果无效（∞ 或 NaN）".to_string());
            }
            Ok(result)
        } else {
            Ok(base)
        }
    }

    // primary := number | '(' expr ')' | func '(' args ')' | constant/variable
    fn parse_primary(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Token::Number(v)) => Ok(v),
            Some(Token::Op('(')) => {
                let v = self.parse_expr()?;
                self.expect_op(')')?;
                Ok(v)
            }
            Some(Token::Ident(name)) => {
                // 函数调用
                if self.eat_op('(') {
                    let mut args = Vec::new();
                    if !self.eat_op(')') {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.eat_op(',') {
                                continue;
                            }
                            self.expect_op(')')?;
                            break;
                        }
                    }
                    return call_function(&name, &args);
                }
                // 常量 / 变量
                match name.as_str() {
                    "pi" => return Ok(std::f64::consts::PI),
                    "e" => return Ok(std::f64::consts::E),
                    "tau" => return Ok(std::f64::consts::TAU),
                    _ => {}
                }
                self.vars
                    .get(&name)
                    .copied()
                    .ok_or_else(|| format!("未知变量: {}", name))
            }
            Some(Token::Op(o)) => Err(format!("意外的运算符 '{}'", o)),
            None => Err("表达式不完整".to_string()),
        }
    }

    fn finish(&self, value: f64) -> Result<f64, String> {
        if self.pos < self.tokens.len() {
            return Err("表达式多余的部分无法解析".to_string());
        }
        if !value.is_finite() {
            return Err("结果无效（∞ 或 NaN）".to_string());
        }
        Ok(value)
    }
}

fn call_function(name: &str, args: &[f64]) -> Result<f64, String> {
    let arity_err = |expected: &str| format!("函数 {} 参数数量错误（{}）", name, expected);
    let value = match name {
        "sin" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).sin(),
        "cos" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).cos(),
        "tan" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).tan(),
        "asin" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).asin(),
        "acos" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).acos(),
        "atan" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).atan(),
        "sqrt" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).sqrt(),
        "cbrt" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).cbrt(),
        "ln" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).ln(),
        "log" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).log10(),
        "log2" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).log2(),
        "abs" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).abs(),
        "round" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).round(),
        "floor" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).floor(),
        "ceil" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).ceil(),
        "exp" => (args.get(0).ok_or_else(|| arity_err("1 个"))?).exp(),
        "min" => {
            let a = args.get(0).ok_or_else(|| arity_err("至少 1 个"))?;
            args.iter().skip(1).fold(*a, |m, v| m.min(*v))
        }
        "max" => {
            let a = args.get(0).ok_or_else(|| arity_err("至少 1 个"))?;
            args.iter().skip(1).fold(*a, |m, v| m.max(*v))
        }
        "pow" => {
            let base = args.get(0).ok_or_else(|| arity_err("2 个"))?;
            let exp = args.get(1).ok_or_else(|| arity_err("2 个"))?;
            base.powf(*exp)
        }
        _ => return Err(format!("未知函数: {}", name)),
    };
    if !value.is_finite() {
        return Err(format!("函数 {} 结果无效（∞ 或 NaN）", name));
    }
    Ok(value)
}

/// 单表达式求值（无变量上下文），成功返回格式化结果
fn eval_expression(expr: &str, vars: &HashMap<String, f64>) -> Result<f64, String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err("表达式为空".to_string());
    }
    let tokens = tokenize(trimmed)?;
    if tokens.is_empty() {
        return Err("表达式为空".to_string());
    }
    let mut parser = Parser {
        tokens,
        pos: 0,
        vars,
    };
    let value = parser.parse_expr()?;
    parser.finish(value)
}

/// 数值格式化：整数不带小数点，小数最多保留 10 位并去尾零
fn format_value(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.10}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

// ===== 命令与数据 =====

#[derive(Debug, Clone, Serialize)]
pub struct CalcLineResult {
    pub input: String,
    pub ok: bool,
    pub value: Option<String>,
    pub error: Option<String>,
    pub is_assignment: bool,
    pub var_name: Option<String>,
}

/// 判断是否为合法变量名
fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 整页稿纸逐行求值：支持 "变量 = 表达式" 赋值行与 ans（上一个成功结果）
fn eval_scratchpad(lines: &[String]) -> Vec<CalcLineResult> {
    let mut vars: HashMap<String, f64> = HashMap::new();
    let mut results = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 赋值行：name = expr
        if let Some(eq) = trimmed.find('=') {
            let name = trimmed[..eq].trim();
            let rest = trimmed[eq + 1..].trim();
            if !name.is_empty() && is_ident(name) && !rest.is_empty() {
                let result = match eval_expression(rest, &vars) {
                    Ok(v) => {
                        if name != "ans" {
                            vars.insert(name.to_string(), v);
                        }
                        vars.insert("ans".to_string(), v);
                        CalcLineResult {
                            input: trimmed.to_string(),
                            ok: true,
                            value: Some(format_value(v)),
                            error: None,
                            is_assignment: true,
                            var_name: Some(name.to_string()),
                        }
                    }
                    Err(e) => CalcLineResult {
                        input: trimmed.to_string(),
                        ok: false,
                        value: None,
                        error: Some(e),
                        is_assignment: true,
                        var_name: Some(name.to_string()),
                    },
                };
                results.push(result);
                continue;
            }
        }

        match eval_expression(trimmed, &vars) {
            Ok(v) => {
                vars.insert("ans".to_string(), v);
                results.push(CalcLineResult {
                    input: trimmed.to_string(),
                    ok: true,
                    value: Some(format_value(v)),
                    error: None,
                    is_assignment: false,
                    var_name: None,
                });
            }
            Err(e) => results.push(CalcLineResult {
                input: trimmed.to_string(),
                ok: false,
                value: None,
                error: Some(e),
                is_assignment: false,
                var_name: None,
            }),
        }
    }

    results
}

/// 单表达式求值命令
#[command]
pub async fn evaluate_expression(expr: String) -> Result<String, String> {
    let vars = HashMap::new();
    let value = eval_expression(&expr, &vars)?;
    Ok(format_value(value))
}

/// 整页稿纸求值命令
#[command]
pub async fn evaluate_scratchpad(lines: Vec<String>) -> Result<Vec<CalcLineResult>, String> {
    Ok(eval_scratchpad(&lines))
}

/// 加载稿纸历史
#[command]
pub async fn load_calc_lines(app: AppHandle) -> Result<Vec<String>, String> {
    let store = app.store("calc.json").map_err(|e| e.to_string())?;
    if let Some(v) = store.get("lines") {
        if let Ok(lines) = serde_json::from_value::<Vec<String>>(v) {
            return Ok(lines);
        }
    }
    Ok(Vec::new())
}

/// 保存稿纸历史（超限保留最近的行）
#[command]
pub async fn save_calc_lines(lines: Vec<String>, app: AppHandle) -> Result<(), String> {
    let store = app.store("calc.json").map_err(|e| e.to_string())?;
    let mut lines = lines;
    if lines.len() > MAX_CALC_LINES {
        let overflow = lines.len() - MAX_CALC_LINES;
        lines.drain(0..overflow);
    }
    store.set(
        "lines",
        serde_json::to_value(&lines).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| format!("持久化失败: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(expr: &str) -> Result<f64, String> {
        eval_expression(expr, &HashMap::new())
    }

    fn eval_str(expr: &str) -> String {
        format_value(eval(expr).unwrap())
    }

    #[test]
    fn test_basic_operators() {
        assert_eq!(eval_str("1 + 2"), "3");
        assert_eq!(eval_str("2 + 3 * 4"), "14");
        assert_eq!(eval_str("(2 + 3) * 4"), "20");
        assert_eq!(eval_str("10 / 4"), "2.5");
        assert_eq!(eval_str("7 % 3"), "1");
        assert_eq!(eval_str("0.1 + 0.2"), "0.3");
    }

    #[test]
    fn test_power_and_unary() {
        assert_eq!(eval_str("2 ^ 10"), "1024");
        assert_eq!(eval_str("2 ** 10"), "1024");
        // 右结合：2^3^2 = 2^9
        assert_eq!(eval_str("2 ^ 3 ^ 2"), "512");
        assert_eq!(eval_str("-5 + 3"), "-2");
        assert_eq!(eval_str("2 ^ -2"), "0.25");
        assert_eq!(eval_str("--5"), "5");
    }

    #[test]
    fn test_percent_suffix() {
        assert_eq!(eval_str("50%"), "0.5");
        assert_eq!(eval_str("50% * 200"), "100");
        assert_eq!(eval_str("200 * 5%"), "10");
        // 带空格的 % 是取模
        assert_eq!(eval_str("7 % 3"), "1");
    }

    #[test]
    fn test_functions_and_constants() {
        assert_eq!(eval_str("sqrt(16)"), "4");
        assert_eq!(eval_str("pow(2, 10)"), "1024");
        assert_eq!(eval_str("min(3, 1, 2)"), "1");
        assert_eq!(eval_str("max(3, 1, 2)"), "3");
        assert_eq!(eval_str("abs(-8)"), "8");
        assert_eq!(eval_str("round(2.5)"), "3");
        assert_eq!(eval_str("floor(2.9)"), "2");
        assert_eq!(eval_str("ceil(2.1)"), "3");
        assert_eq!(eval_str("log(1000)"), "3");
        assert_eq!(eval_str("log2(8)"), "3");
        assert!(eval_str("sin(0)").eq("0"));
        assert_eq!(eval_str("round(pi * 100) / 100"), "3.14");
        // tau / 2 == pi
        let tau_half = eval("tau / 2").unwrap();
        let pi = eval("pi").unwrap();
        assert!((tau_half - pi).abs() < 1e-12);
        // e 参与运算
        assert!((eval("e - 2").unwrap() - std::f64::consts::E + 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_errors() {
        assert!(eval("1 / 0").is_err());
        assert!(eval("1 % 0").is_err());
        assert!(eval("unknown_var + 1").is_err());
        assert!(eval("foo(1)").is_err());
        assert!(eval("(1 + 2").is_err());
        assert!(eval("1 + 2)").is_err());
        assert!(eval("").is_err());
        assert!(eval("1 @ 2").is_err());
        assert!(eval("sqrt(-1)").is_err()); // NaN
        assert!(eval("1 2").is_err()); // 多余部分
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(3.0), "3");
        assert_eq!(format_value(-0.0), "0");
        assert_eq!(format_value(2.5), "2.5");
        assert_eq!(format_value(0.30000000000000004), "0.3");
        assert_eq!(format_value(1e10 + 0.5), "10000000000.5");
    }

    #[test]
    fn test_scratchpad_variables_and_ans() {
        let lines: Vec<String> = vec![
            "a = 1 + 2".into(),
            "b = a * 2".into(),
            "b + 1".into(),
            "ans + 10".into(),
        ];
        let results = eval_scratchpad(&lines);
        assert_eq!(results.len(), 4);
        assert!(results[0].ok && results[0].value.as_deref() == Some("3"));
        assert!(results[1].ok && results[1].value.as_deref() == Some("6"));
        assert!(results[2].ok && results[2].value.as_deref() == Some("7"));
        assert!(results[3].ok && results[3].value.as_deref() == Some("17"));
    }

    #[test]
    fn test_scratchpad_error_line_and_continue() {
        let lines: Vec<String> = vec![
            "x = 10".into(),
            "x / 0".into(),
            "x * 2".into(),
            "".into(), // 空行跳过
        ];
        let results = eval_scratchpad(&lines);
        assert_eq!(results.len(), 3);
        assert!(results[0].ok);
        assert!(!results[1].ok);
        assert!(results[1].error.as_deref().unwrap().contains("除数"));
        assert!(results[2].ok && results[2].value.as_deref() == Some("20"));
    }

    #[test]
    fn test_scratchpad_invalid_assignment_falls_back_to_expr() {
        // 含 = 但左侧不是合法变量名 → 按普通表达式解析（报错）
        let lines: Vec<String> = vec!["1a = 5".into()];
        let results = eval_scratchpad(&lines);
        assert_eq!(results.len(), 1);
        assert!(!results[0].ok);
    }
}
