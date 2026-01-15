use super::Interpreter;
use crate::tokens::{Token, TokenType};

impl Interpreter {
    pub fn handle_solve_constructor(&mut self, i: &mut usize, tokens: &Vec<Token>) -> String {
        *i += 2; // Skip S and [
        let mut inner_tokens = Vec::new();
        let mut bracket_depth = 1;

        while *i < tokens.len() {
            let t = &tokens[*i];
            if t.token_type == TokenType::LBracket {
                bracket_depth += 1;
            }
            if t.token_type == TokenType::RBracket {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    break;
                }
            }
            inner_tokens.push(t.clone());
            *i += 1;
        }

        if *i < tokens.len() && tokens[*i].token_type == TokenType::RBracket {
            *i += 1;
        }

        let result = self.evaluate_math_recursive(&inner_tokens);
        if result.fract() == 0.0 {
            format!("{:.0}", result)
        } else {
            result.to_string()
        }
    }

    fn resolve_val(&self, token: &TokenType) -> f64 {
        match token {
            TokenType::Number(n) => *n,
            TokenType::SpeedVal(n) => *n,
            TokenType::Identifier(v) => {
                let val_str = self.get_variable(v).unwrap_or_else(|| v.clone());
                val_str.parse::<f64>().unwrap_or(0.0)
            }
            _ => 0.0,
        }
    }

    pub fn evaluate_math_recursive(&mut self, tokens: &[Token]) -> f64 {
        if tokens.is_empty() {
            return 0.0;
        }

        // 1. Handle Parentheses and Functions
        let mut flattened = Vec::new();
        let mut j = 0;
        while j < tokens.len() {
            match &tokens[j].token_type {
                TokenType::LParen => {
                    let start = j + 1;
                    let mut depth = 1;
                    let mut k = j + 1;
                    while k < tokens.len() {
                        if tokens[k].token_type == TokenType::LParen {
                            depth += 1;
                        }
                        if tokens[k].token_type == TokenType::RParen {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        k += 1;
                    }
                    let sub_result = self.evaluate_math_recursive(&tokens[start..k]);
                    flattened.push(TokenType::Number(sub_result));
                    j = k + 1;
                }
                TokenType::Sqrt
                | TokenType::Sin
                | TokenType::Cos
                | TokenType::Tan
                | TokenType::Abs
                | TokenType::Log => {
                    let func = tokens[j].token_type.clone();
                    j += 1;
                    let mut val = 0.0;
                    if j < tokens.len() {
                        if tokens[j].token_type == TokenType::LParen {
                            let start = j + 1;
                            let mut depth = 1;
                            let mut k = j + 1;
                            while k < tokens.len() {
                                if tokens[k].token_type == TokenType::LParen {
                                    depth += 1;
                                }
                                if tokens[k].token_type == TokenType::RParen {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                k += 1;
                            }
                            val = self.evaluate_math_recursive(&tokens[start..k]);
                            j = k + 1;
                        } else {
                            val = self.resolve_val(&tokens[j].token_type);
                            j += 1;
                        }
                    }
                    let res = match func {
                        TokenType::Sqrt => val.sqrt(),
                        TokenType::Sin => val.to_radians().sin(),
                        TokenType::Cos => val.to_radians().cos(),
                        TokenType::Tan => val.to_radians().tan(),
                        TokenType::Abs => val.abs(),
                        TokenType::Log => val.log10(),
                        _ => 0.0,
                    };
                    flattened.push(TokenType::Number(res));
                }
                TokenType::Minimum | TokenType::Maximum => {
                    let func = tokens[j].token_type.clone();
                    j += 1;
                    let val1 = if j < tokens.len() {
                        self.resolve_val(&tokens[j].token_type)
                    } else {
                        0.0
                    };
                    if j < tokens.len() {
                        j += 1;
                    }
                    if j < tokens.len() {
                        if let TokenType::Identifier(s) = &tokens[j].token_type {
                            if s == "and" {
                                j += 1;
                            }
                        }
                    }
                    let val2 = if j < tokens.len() {
                        self.resolve_val(&tokens[j].token_type)
                    } else {
                        0.0
                    };
                    if j < tokens.len() {
                        j += 1;
                    }
                    let res = if func == TokenType::Minimum {
                        val1.min(val2)
                    } else {
                        val1.max(val2)
                    };
                    flattened.push(TokenType::Number(res));
                }
                TokenType::PI => {
                    flattened.push(TokenType::Number(std::f64::consts::PI));
                    j += 1;
                }
                TokenType::Euler => {
                    flattened.push(TokenType::Number(std::f64::consts::E));
                    j += 1;
                }
                TokenType::BigTick => {
                    flattened.push(TokenType::Number(
                        self.start_time.elapsed().as_millis() as f64
                    ));
                    j += 1;
                }
                TokenType::BigDelta => {
                    flattened.push(TokenType::Number(self.current_dt as f64));
                    j += 1;
                }
                TokenType::Dollar => {
                    j += 1;
                }
                TokenType::Len => {
                    j += 1;
                    if j < tokens.len() {
                        let mut temp_j = j;
                        let val_raw = self.get_complex_value(&mut temp_j, &tokens.to_vec());
                        let val = self.interpolate_string(&val_raw);
                        // println!("MATH DEBUG: len raw='{}' interp='{}' res={}", val_raw, val, val.len());
                        flattened.push(TokenType::Number(val.len() as f64));
                        j = temp_j;
                    }
                }
                _ => {
                    flattened.push(tokens[j].token_type.clone());
                    j += 1;
                }
            }
        }

        // 1.5 Pass 0: Exponents (^) and Remainder (mod)
        let mut pass0 = Vec::new();
        let mut e = 0;
        while e < flattened.len() {
            match &flattened[e] {
                TokenType::Caret | TokenType::Remainder => {
                    let op = flattened[e].clone();
                    let prev_val = if let Some(t) = pass0.pop() {
                        self.resolve_val(&t)
                    } else {
                        0.0
                    };
                    if e + 1 < flattened.len() {
                        let next_val = self.resolve_val(&flattened[e + 1]);
                        let res = if let TokenType::Caret = op {
                            prev_val.powf(next_val)
                        } else if next_val == 0.0 {
                            0.0
                        } else {
                            prev_val % next_val
                        };
                        pass0.push(TokenType::Number(res));
                        e += 2;
                        continue;
                    }
                }
                other => pass0.push(other.clone()),
            }
            e += 1;
        }

        // 2. Pass 1: Multiplication and Division
        let mut pass1 = Vec::new();
        let mut m = 0;
        while m < pass0.len() {
            match &pass0[m] {
                TokenType::Star | TokenType::Slash => {
                    let op_type = pass0[m].clone();
                    let prev_val = if let Some(t) = pass1.pop() {
                        self.resolve_val(&t)
                    } else {
                        0.0
                    };
                    if m + 1 < pass0.len() {
                        let next_val = self.resolve_val(&pass0[m + 1]);
                        let res = if let TokenType::Star = op_type {
                            prev_val * next_val
                        } else if next_val == 0.0 {
                            self.last_bug_found = true;
                            self.last_bug_type = String::from("DivisionByZero");
                            0.0
                        } else {
                            prev_val / next_val
                        };
                        pass1.push(TokenType::Number(res));
                        m += 2;
                        continue;
                    }
                }
                other => pass1.push(other.clone()),
            }
            m += 1;
        }

        // 3. Pass 2: Addition and Subtraction
        let mut final_result = 0.0;
        let mut current_op = TokenType::Plus;
        for item in pass1 {
            match item {
                TokenType::Plus => current_op = TokenType::Plus,
                TokenType::Minus => current_op = TokenType::Minus,
                other => {
                    let n = self.resolve_val(&other);
                    match current_op {
                        TokenType::Plus => final_result += n,
                        TokenType::Minus => final_result -= n,
                        _ => {}
                    }
                }
            }
        }
        // if final_result.abs() < 10.0 { println!("MATH DEBUG: result={}", final_result); }
        final_result
    }
}
