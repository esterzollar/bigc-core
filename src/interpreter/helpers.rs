use super::Interpreter;
use crate::tokens::{Token, TokenType};
use serde_json::Value;
use std::io::{self, Write};

impl Interpreter {
    pub fn get_token_len(&self, token: &Token) -> usize {
        match &token.token_type {
            TokenType::String(s) => s.len() + 2,
            TokenType::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n).len()
                } else {
                    n.to_string().len()
                }
            }
            TokenType::Identifier(s) => s.len(),
            TokenType::PythonCode(s) => s.len(),
            TokenType::Char(_) => 1,
            _ => 1,
        }
    }

    pub fn validate_at_strictness(&self, i: usize, tokens: &Vec<Token>) -> bool {
        if tokens[i].token_type != TokenType::At {
            return true;
        }
        if i > 0 && tokens[i].line == tokens[i - 1].line {
            let prev_len = self.get_token_len(&tokens[i - 1]);
            if tokens[i].column <= tokens[i - 1].column + prev_len {
                println!(
                    "Big Error: Missing space before '@'! (Line {})",
                    tokens[i].line
                );
                return false;
            }
        }
        if i + 1 < tokens.len()
            && tokens[i + 1].column != tokens[i].column + 1 {
                println!("Big Error: The target must be attached directly to '@'. No spaces allowed! (Line {})", tokens[i].line);
                return false;
            }
        true
    }

    pub fn handle_set_as_multiple(
        &mut self,
        i: &mut usize,
        tokens: &Vec<Token>,
        results: Vec<String>,
    ) {
        let mut j = *i + 1;
        if j < tokens.len() && tokens[j].token_type == TokenType::Ampersand {
            j += 1;
            if j < tokens.len() && tokens[j].token_type == TokenType::Set {
                j += 1;
                if j < tokens.len() && tokens[j].token_type == TokenType::As {
                    j += 1;
                    let is_list = if j < tokens.len() && tokens[j].token_type == TokenType::List {
                        j += 1;
                        true
                    } else {
                        false
                    };
                    if is_list {
                        let mut json_arr = String::from("[");
                        for (idx, val) in results.iter().enumerate() {
                            if idx > 0 {
                                json_arr.push(',');
                            }
                            json_arr.push_str(&format!("\"{}\"", val.replace("\"", "\\\"")));
                        }
                        json_arr.push(']');
                        let target = self.extract_braced_name(&mut j, tokens);
                        if !target.is_empty() {
                            self.set_variable(target, json_arr);
                        }
                    } else {
                        let mut res_idx = 0;
                        while j < tokens.len() && res_idx < results.len() {
                            if tokens[j].token_type == TokenType::LBrace {
                                let target = self.extract_braced_name(&mut j, tokens);
                                if !target.is_empty() {
                                    self.set_variable(target, results[res_idx].clone());
                                }
                                res_idx += 1;
                            } else {
                                // Assume it's an Identifier or Keyword used as name
                                let target = self.get_token_raw_name(&tokens[j]);
                                if !target.is_empty() && !target.contains('(') {
                                    self.set_variable(target, results[res_idx].clone());
                                    j += 1;
                                    res_idx += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    *i = j - 1;
                }
            }
        }
    }

    pub fn evaluate_condition(&self, tokens: &[Token]) -> bool {
        if tokens.is_empty() {
            return true;
        }

        // 1. Handle special cases (btnclick, hover, click, keydown)
        if tokens.len() == 1 {
            match tokens[0].token_type {
                TokenType::BtnClick => return *self.last_widget_clicked.read().unwrap(),
                TokenType::Hover => return self.last_condition_met,
                TokenType::Click => return *self.global_clicked.read().unwrap(),
                _ => {}
            }
        }

        // Handle: if keydown "A"
        if tokens.len() == 2 {
            if tokens[0].token_type == TokenType::KeyDown {
                let key_name = self.get_token_value(&tokens[1]).to_lowercase();
                if let Ok(keys) = self.keys_down.read() {
                    return *keys.get(&key_name).unwrap_or(&false);
                }
                return false;
            }
            // Handle: if hover "Tag"
            if tokens[0].token_type == TokenType::Hover {
                let tag = self.get_token_value(&tokens[1]);
                if let Ok(tags) = self.last_hovered_tags.read() {
                    return *tags.get(&tag).unwrap_or(&false);
                }
                return false;
            }
            // Handle: if click "Tag"
            if tokens[0].token_type == TokenType::Click {
                let tag = self.get_token_value(&tokens[1]);
                if let Ok(tags) = self.last_clicked_tags.read() {
                    return *tags.get(&tag).unwrap_or(&false);
                }
                return false;
            }
            // Handle: if press "Tag"
            if tokens[0].token_type == TokenType::Press {
                let tag = self.get_token_value(&tokens[1]);
                if let Ok(tags) = self.last_pressed_tags.read() {
                    return *tags.get(&tag).unwrap_or(&false);
                }
                return false;
            }
            // Handle: if drag "Tag"
            if tokens[0].token_type == TokenType::Drag {
                let tag = self.get_token_value(&tokens[1]);
                if let Ok(tags) = self.last_dragged_tags.read() {
                    return tags.contains_key(&tag);
                }
                return false;
            }
        }

        // Handle: if any bug found
        if tokens.len() == 3
            && tokens[0].token_type == TokenType::Any
                && tokens[1].token_type == TokenType::Bug
                && tokens[2].token_type == TokenType::Found
            {
                return self.last_bug_found;
            }

        // 2. Resolve logic (Stop at first operator)
        let mut op_pos = None;
        let mut op_type = None;
        for (idx, token) in tokens.iter().enumerate() {
            match token.token_type {
                TokenType::Greater
                | TokenType::Less
                | TokenType::GreaterEqual
                | TokenType::LessEqual
                | TokenType::NotEqual
                | TokenType::Assign => {
                    op_pos = Some(idx);
                    op_type = Some(token.token_type.clone());
                    break;
                }
                _ => {}
            }
        }

        if let (Some(pos), Some(op)) = (op_pos, op_type) {
            let left_raw = self.get_tokens_raw_value(&tokens[0..pos]);
            let right_raw = self.get_tokens_raw_value(&tokens[pos + 1..]);

            let left_num = left_raw.parse::<f64>();
            let right_num = right_raw.parse::<f64>();

            if let (Ok(ln), Ok(rn)) = (left_num, right_num) {
                match op {
                    TokenType::Greater => ln > rn,
                    TokenType::Less => ln < rn,
                    TokenType::GreaterEqual => ln >= rn,
                    TokenType::LessEqual => ln <= rn,
                    TokenType::NotEqual => (ln - rn).abs() > 0.00001,
                    TokenType::Assign => (ln - rn).abs() < 0.00001,
                    _ => false,
                }
            } else {
                match op {
                    TokenType::Greater => left_raw > right_raw,
                    TokenType::Less => left_raw < right_raw,
                    TokenType::GreaterEqual => left_raw >= right_raw,
                    TokenType::LessEqual => left_raw <= right_raw,
                    TokenType::NotEqual => left_raw != right_raw,
                    TokenType::Assign => left_raw == right_raw,
                    _ => false,
                }
            }
        } else {
            self.evaluate_expression(tokens) > 0.0
        }
    }

    pub fn get_tokens_raw_value(&self, tokens: &[Token]) -> String {
        let mut result = String::new();
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].token_type == TokenType::Dollar
                && i + 1 < tokens.len() {
                    let next = &tokens[i + 1];
                    let var_name = self.get_token_raw_name(next);

                    // Try to resolve variable, if not found, keep literal $Name
                    // Note: Use the RAW name for lookup to support case-fallback in get_variable
                    if let Some(val) = self.get_variable(&var_name) {
                        result.push_str(&val);
                        i += 2;
                        continue;
                    }
                }
            result.push_str(&self.get_token_value(&tokens[i]));
            i += 1;
        }
        result
    }

    pub fn evaluate_expression(&self, tokens: &[Token]) -> f64 {
        let mut val = 0.0;
        let mut op = TokenType::Plus;
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i].token_type {
                TokenType::Plus => op = TokenType::Plus,
                TokenType::Minus => op = TokenType::Minus,
                TokenType::Number(n) => match op {
                    TokenType::Plus => val += n,
                    TokenType::Minus => val -= n,
                    _ => {}
                },
                TokenType::Identifier(v) => {
                    let num = self
                        .get_variable(v)
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    match op {
                        TokenType::Plus => val += num,
                        TokenType::Minus => val -= num,
                        _ => {}
                    }
                }
                _ => {}
            }
            i += 1;
        }
        val
    }

    pub fn get_complex_value(&mut self, i: &mut usize, tokens: &Vec<Token>) -> String {
        if *i >= tokens.len() {
            return String::new();
        }

        if tokens[*i].token_type == TokenType::At || tokens[*i].token_type == TokenType::AtWord {
            *i += 1;
        }
        if *i >= tokens.len() {
            return String::new();
        }

        if tokens[*i].token_type == TokenType::Len {
            *i += 1;
            let mut has_brace = false;
            if *i < tokens.len() && tokens[*i].token_type == TokenType::LBrace {
                has_brace = true;
                *i += 1;
            }

            let val_raw = self.get_complex_value(i, tokens);

            if has_brace && *i < tokens.len() && tokens[*i].token_type == TokenType::RBrace {
                *i += 1;
            }

            let val = self.interpolate_string(&val_raw);
            return val.len().to_string();
        }
        if tokens[*i].token_type == TokenType::Dollar
            && *i + 1 < tokens.len() {
                let next_token = &tokens[*i + 1];

                // Allow ANY token to be treated as a variable name after $
                let var_name = self.get_token_raw_name(next_token);
                let val = self
                    .get_variable(&var_name)
                    .unwrap_or_else(|| format!("${}", var_name));
                *i += 2;
                return val;
            }

        // Warp Constructor: w(...)
        let is_warp_call = match &tokens[*i].token_type {
            TokenType::Warp => true,
            TokenType::Identifier(func) if func == "w" || func == "warp" => true,
            _ => false,
        };
        if is_warp_call && *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::LParen {
            return self.handle_warp_constructor(i, tokens);
        }

        // Solver Constructor: S[...]
        if tokens[*i].token_type == TokenType::Solve
            && *i + 1 < tokens.len()
            && tokens[*i + 1].token_type == TokenType::LBracket
        {
            return self.handle_solve_constructor(i, tokens);
        }

        // --- AUTO-INTERPOLATE STRINGS ---
        if let TokenType::String(ref s) = tokens[*i].token_type {
            let val = self.interpolate_string(s);
            *i += 1;
            return val;
        }

        let val = self.get_token_value(&tokens[*i]);
        *i += 1;
        val
    }

    pub fn handle_warp_constructor(&mut self, i: &mut usize, tokens: &Vec<Token>) -> String {
        let mut warp_content = String::new();
        *i += 2;
        let mut depth = 1;
        while *i < tokens.len() {
            if tokens[*i].token_type == TokenType::LParen {
                depth += 1;
            }
            if tokens[*i].token_type == TokenType::RParen {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }

            warp_content.push_str(&self.get_token_raw_name(&tokens[*i]));

            // Smart Space: Only if there was a space in the source
            if *i + 1 < tokens.len() {
                let current = &tokens[*i];
                let next = &tokens[*i + 1];

                if next.line == current.line {
                    let current_len = self.get_token_len(current);
                    if next.column > current.column + current_len {
                        warp_content.push(' ');
                    }
                } else if next.line > current.line {
                    // New line usually implies separation
                    warp_content.push(' ');
                }
            }

            *i += 1;
        }
        *i += 1;
        self.interpolate_string(warp_content.trim())
    }

    pub fn get_token_value(&self, token: &Token) -> String {
        match &token.token_type {
            TokenType::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            TokenType::Identifier(v) => self.get_variable(v).unwrap_or_else(|| v.clone()),
            TokenType::String(s) => s.clone(),
            TokenType::Assign => String::from("="),
            TokenType::Ampersand => String::from("&"),
            TokenType::Plus => String::from("+"),
            TokenType::Minus => String::from("-"),
            TokenType::Star => String::from("*"),
            TokenType::Slash => String::from("/"),
            TokenType::GreaterEqual => String::from(">="),
            TokenType::LessEqual => String::from("<="),
            TokenType::NotEqual => String::from("!= "),
            TokenType::Dollar => String::from("$"),
            TokenType::Dot => String::from("."),
            TokenType::Colon => String::from(":"),
            TokenType::LBracket => String::from("["),
            TokenType::RBracket => String::from("]"),
            TokenType::LParen => String::from("("),
            TokenType::RParen => String::from(")"),
            TokenType::Greater => String::from(">"),
            TokenType::Less => String::from("<"),
            TokenType::At | TokenType::AtWord => String::from("at"),
            TokenType::Char(c) => c.to_string(),
            _ => {
                let name = format!("{:?}", token.token_type).to_lowercase();
                if let Some(val) = self.get_variable(&name) {
                    val
                } else {
                    name
                }
            }
        }
    }

    pub fn interpolate_string(&self, text: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '$' {
                result.push('$');
                i += 2;
            } else if chars[i] == '$' {
                i += 1;
                if i < chars.len() && chars[i] == '{' {
                    i += 1;
                    let mut var_name = String::new();
                    while i < chars.len() && chars[i] != '}' {
                        var_name.push(chars[i]);
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '}' {
                        i += 1;
                    }
                    result.push_str(
                        &self
                            .get_variable(&var_name)
                            .unwrap_or_else(|| format!("${{{}}}", var_name)),
                    );
                } else {
                    let mut var_name = String::new();
                    while i < chars.len() {
                        let c = chars[i];
                        if c.is_alphanumeric() || c == '_' {
                            var_name.push(c);
                            i += 1;
                        } else if c == '.' && i + 1 < chars.len() && chars[i + 1].is_alphanumeric()
                        {
                            // Dot notation: only if followed by property name
                            var_name.push(c);
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    let val = self.get_variable(&var_name);
                    result.push_str(&val.unwrap_or_else(|| format!("${}", var_name)));
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    pub fn print_interpolated(&self, text: &str, newline: bool) {
        let result = self.interpolate_string(text);
        if newline {
            println!("{}", result);
        } else {
            print!("{}", result);
            io::stdout().flush().unwrap();
        }
    }

    pub fn draw_big_box(&self, title: &str, lines: Vec<String>) {
        println!("+--- {} ---+", title);
        for line in lines {
            println!("| {}", line);
        }
        println!("+----------+");
    }

    pub fn validate_syntax(&self, tokens: &Vec<Token>) -> bool {
        for token in tokens {
            if let TokenType::Identifier(ref name) = token.token_type {
                if name == "Val" {
                    println!("Big Error: 'Val' is a reserved internal keyword!");
                    println!(
                        "  > Line {}: Do not use 'Val' as a variable name.",
                        token.line
                    );
                    println!("  > Fix: Rename it to 'Value', 'Num', or 'MyVal'.");
                    return false;
                }
            }
        }
        true
    }

    pub fn parse_json_list(&self, raw: &str) -> Vec<String> {
        let trimmed = raw.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return trimmed[1..trimmed.len() - 1]
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .collect();
        }
        Vec::new()
    }

    pub fn parse_json_map_keys(&self, raw: &str) -> Vec<String> {
        if let Ok(Value::Object(map)) = serde_json::from_str(raw) {
            return map.keys().cloned().collect();
        }
        Vec::new()
    }

    pub fn get_map_value(&self, raw: &str, key: &str) -> String {
        if let Ok(Value::Object(map)) = serde_json::from_str(raw) {
            if let Some(val) = map.get(key) {
                return match val {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => val.to_string(),
                };
            }
        }
        String::from("nothing")
    }
    pub fn get_token_raw_name(&self, token: &Token) -> String {
        match &token.token_type {
            TokenType::Identifier(s) => s.clone(),
            TokenType::String(s) => s.clone(),
            TokenType::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            TokenType::Char(c) => c.to_string(),
            TokenType::Colon => String::from(":"),
            TokenType::Assign => String::from("="),
            TokenType::Ampersand => String::from("&"),
            TokenType::Plus => String::from("+"),
            TokenType::Minus => String::from("-"),
            TokenType::Star => String::from("*"),
            TokenType::Slash => String::from("/"),
            TokenType::Dollar => String::from("$"),
            TokenType::Dot => String::from("."),
            TokenType::LParen => String::from("("),
            TokenType::RParen => String::from(")"),
            TokenType::LBracket => String::from("["),
            TokenType::RBracket => String::from("]"),
            TokenType::LBrace => String::from("{"),
            TokenType::RBrace => String::from("}"),
            TokenType::Greater => String::from(">"),
            TokenType::Less => String::from("<"),
            _ => {
                let debug_name = format!("{:?}", token.token_type);
                if debug_name.contains('(') {
                    self.get_token_value(token)
                } else {
                    debug_name.to_lowercase()
                }
            }
        }
    }
    pub fn extract_braced_name(&self, i: &mut usize, tokens: &Vec<Token>) -> String {
        if tokens[*i].token_type == TokenType::LBrace {
            *i += 1;
            let name = self.get_token_raw_name(&tokens[*i]);
            *i += 2; // Consume Name and RBrace
            name
        } else {
            let name = self.get_token_raw_name(&tokens[*i]);
            *i += 1; // Consume Name
            name
        }
    }
}
