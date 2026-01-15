use super::Interpreter;
use crate::lexer::Lexer;
use crate::tokens::{Token, TokenType};
use std::fs;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

impl Interpreter {
    pub fn handle_action(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        let start_index = *i;
        match &tokens[*i].token_type {
            TokenType::Print => {
                *i += 1;
                if *i < tokens.len() {
                    let mut is_update = false;
                    if tokens[*i].token_type == TokenType::Update {
                        *i += 1;
                        is_update = true;
                    }

                    let mut output = String::new();
                    loop {
                        let val = self.get_complex_value(i, tokens);
                        output.push_str(&self.interpolate_string(&val));

                        if *i < tokens.len() && tokens[*i].token_type == TokenType::Ampersand {
                            *i += 1;
                        } else {
                            break;
                        }
                    }

                    if is_update {
                        print!("\r{}", output);
                        io::stdout().flush().unwrap();
                    } else {
                        println!("{}", output);
                    }
                }
            }
            TokenType::Wait => {
                *i += 1;
                if *i < tokens.len() {
                    if let TokenType::Number(n) = &tokens[*i].token_type {
                        let seconds = *n;
                        *i += 1;
                        if *i < tokens.len() {
                            let is_s = match &tokens[*i].token_type {
                                TokenType::Identifier(s) if s == "s" => true,
                                TokenType::Solve => true, // 's' is an alias for 'solve'
                                _ => false,
                            };
                            if is_s {
                                thread::sleep(Duration::from_secs_f64(seconds));
                            }
                        }
                    }
                }
            }
            TokenType::Take => {
                *i += 1;
                if *i < tokens.len() {
                    if tokens[*i].token_type == TokenType::Body {
                        let body = self
                            .variables
                            .read()
                            .unwrap()
                            .get("RequestBody")
                            .cloned()
                            .unwrap_or(String::new());
                        self.handle_set_as_multiple(i, tokens, vec![body]);
                    } else if tokens[*i].token_type
                        == TokenType::Identifier(String::from("response"))
                    {
                        let mut j = *i + 1;
                        let mut found_wait = false;
                        while j < tokens.len() {
                            if tokens[j].token_type == TokenType::Wait
                                && j + 2 < tokens.len()
                                    && tokens[j + 1].token_type == TokenType::Type
                                    && tokens[j + 2].token_type == TokenType::Export
                                {
                                    found_wait = true;
                                    break;
                                }
                            j += 1;
                        }
                        if found_wait {
                            io::stdout().flush().unwrap();
                            let mut input = String::new();
                            io::stdin().read_line(&mut input).expect("Fail");
                            self.set_variable(String::from("export"), input.trim().to_string());
                            *i = j + 2;
                        }
                    }
                }
            }
            TokenType::Ask => {
                *i += 1; // Skip 'ask'
                if *i < tokens.len() && tokens[*i].token_type == TokenType::Input {
                    *i += 1; // Skip 'input'
                    if *i < tokens.len() {
                        if let TokenType::String(prompt) = &tokens[*i].token_type {
                            self.print_interpolated(prompt, false);
                            io::stdout().flush().unwrap();
                            *i += 1;
                        }
                    }
                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read input");
                    let result = input.trim().to_string();
                    *i -= 1;
                    self.handle_set_as_multiple(i, tokens, vec![result]);
                    *i += 1;
                }
            }
            TokenType::Reset => {
                *i += 1;
                if *i < tokens.len() {
                    let source_raw = self.get_token_value(&tokens[*i]);
                    let source_val = self.interpolate_string(&source_raw);
                    *i += 1;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        *i += 1;
                        if *i < tokens.len() {
                            if let TokenType::Identifier(dest_name) = &tokens[*i].token_type {
                                self.set_variable(dest_name.clone(), source_val);
                            }
                        }
                    } else if *i < tokens.len() && tokens[*i].token_type == TokenType::Ampersand {
                        let path = self.get_variable("RequestPath").unwrap_or_default();
                        let mut found_val = String::from("nothing");
                        if let Some(query_start) = path.find('?') {
                            let query = &path[query_start + 1..];
                            for pair in query.split('&') {
                                let mut parts = pair.split('=');
                                if let Some(key) = parts.next() {
                                    if key == source_val {
                                        found_val = parts.next().unwrap_or("").to_string();
                                        found_val = found_val.replace("+", " ");
                                        break;
                                    }
                                }
                            }
                        }
                        *i -= 1;
                        self.handle_set_as_multiple(i, tokens, vec![found_val]);
                        *i += 1;
                    }
                }
            }
            TokenType::Attach => {
                *i += 1;
                if *i < tokens.len() {
                    // Allow Identifiers OR Keywords (like 'len')
                    let env_name = self.get_token_raw_name(&tokens[*i]);
                    if !env_name.is_empty() {
                        let path =
                            std::path::Path::new("env_lib").join(format!("{}.bigenv", env_name));
                        // println!("DEBUG ATTACH: Loading {:?}", path);
                        if let Ok(env_content) = fs::read_to_string(&path) {
                            let mut env_lexer = Lexer::new(&env_content);
                            self.run(env_lexer.tokenize());
                        }
                    }
                }
            }
            TokenType::UserAgent => {
                *i += 1;
                if *i < tokens.len() {
                    let val = self.get_token_value(&tokens[*i]);
                    self.net.set_user_agent(&val);
                }
            }
            TokenType::Proxy => {
                *i += 1;
                if *i < tokens.len() {
                    let val = self.get_token_value(&tokens[*i]);
                    self.net.set_proxy(&val);
                }
            }
            TokenType::Header => {
                *i += 1;
                if *i < tokens.len() {
                    let key = self.get_token_value(&tokens[*i]);
                    *i += 1;
                    if *i < tokens.len() {
                        let val = self.get_token_value(&tokens[*i]);
                        self.net.add_header(&key, &val);
                    }
                }
            }
            TokenType::Split => {
                *i += 1;
                let source = self.get_token_value(&tokens[*i]);
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::By {
                    *i += 1;
                    let delim = self.get_token_value(&tokens[*i]);
                    let parts: Vec<String> =
                        source.split(&delim).map(|s| s.trim().to_string()).collect();
                    self.handle_set_as_multiple(i, tokens, parts);
                    *i += 1;
                }
            }
            TokenType::Event => {
                *i += 1;
                if *i < tokens.len() {
                    match tokens[*i].token_type {
                        TokenType::Push => {
                            *i += 1;
                            let signal_raw = self.get_token_value(&tokens[*i]);
                            let signal = self.interpolate_string(&signal_raw);
                            if let Ok(mut q) = self.event_queue.write() {
                                q.push_back(signal);
                            }
                        }
                        TokenType::Pop => {
                            let signal = if let Ok(mut q) = self.event_queue.write() {
                                q.pop_front().unwrap_or(String::from("nothing"))
                            } else {
                                String::from("nothing")
                            };
                            self.handle_set_as_multiple(i, tokens, vec![signal]);
                            *i += 1;
                        }
                        _ => {}
                    }
                }
            }
            TokenType::Build => {
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::Task {
                    *i += 1;
                    let cmd_raw = self.get_token_value(&tokens[*i]);
                    let cmd = self.interpolate_string(&cmd_raw);
                    use std::process::Command;
                    let output = if cfg!(target_os = "windows") {
                        Command::new("cmd").args(["/C", &cmd]).output()
                    } else {
                        Command::new("sh").args(["-c", &cmd]).output()
                    };
                    let result = match output {
                        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
                        Err(e) => format!("Build Task Error: {}", e),
                    };
                    self.handle_set_as_multiple(i, tokens, vec![result]);
                    *i += 1;
                }
            }
            TokenType::Pack => {
                *i += 1;
                let map_name = self.extract_braced_name(i, tokens);
                let map_val = self.get_variable(&map_name).unwrap_or(String::from("{}"));
                if *i < tokens.len() && tokens[*i].token_type == TokenType::As {
                    *i += 1;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::Json {
                        if map_val.trim().starts_with('{') {
                            self.handle_set_as_multiple(i, tokens, vec![map_val]);
                            *i += 1;
                        } else {
                            self.last_bug_found = true;
                            self.last_bug_type = String::from("Pack Error: Source is not a Map");
                            self.set_variable("BugType".to_string(), self.last_bug_type.clone());
                        }
                    }
                }
            }
            TokenType::Unpack => {
                *i += 1;
                let json_raw = self.get_complex_value(i, tokens);
                let json_val = self.interpolate_string(&json_raw);
                if *i < tokens.len() && tokens[*i].token_type == TokenType::As {
                    *i += 1;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::Map {
                        if serde_json::from_str::<serde_json::Value>(&json_val).is_ok() {
                            self.handle_set_as_multiple(i, tokens, vec![json_val]);
                            *i += 1;
                        } else {
                            self.last_bug_found = true;
                            self.last_bug_type = String::from("Unpack Error: Invalid JSON String");
                            self.set_variable("BugType".to_string(), self.last_bug_type.clone());
                            self.handle_set_as_multiple(i, tokens, vec![String::from("{}")]);
                            *i += 1;
                        }
                    }
                }
            }
            TokenType::Replace => {
                *i += 1;
                let old_val_raw = self.get_complex_value(i, tokens);
                let old_val = self.interpolate_string(&old_val_raw);
                if *i < tokens.len() && tokens[*i].token_type == TokenType::With {
                    *i += 1;
                    let new_val_raw = self.get_complex_value(i, tokens);
                    let new_val = self.interpolate_string(&new_val_raw);
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        *i += 1;
                        let target_is_braced = tokens[*i].token_type == TokenType::LBrace;
                        let target_name = self.extract_braced_name(i, tokens);
                        if target_is_braced {
                            let current_text =
                                self.get_variable(&target_name).unwrap_or_default();
                            let updated = current_text.replace(&old_val, &new_val);
                            self.set_variable(target_name, updated);
                        } else {
                            self.last_bug_found = false;
                            if let Ok(content) = std::fs::read_to_string(&target_name) {
                                let updated = content.replace(&old_val, &new_val);
                                if let Err(e) = std::fs::write(&target_name, updated) {
                                    self.last_bug_found = true;
                                    self.last_bug_type = format!("File Error: {}", e);
                                }
                            } else {
                                self.last_bug_found = true;
                                self.last_bug_type =
                                    String::from("File Error: Could not read file for replace");
                            }
                        }
                    }
                }
            }
            TokenType::Command => {
                let args: Vec<String> = std::env::args().skip(2).collect();
                self.handle_set_as_multiple(i, tokens, args);
                *i += 1;
            }
            TokenType::Asset => {
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::Load {
                    *i += 1;
                    let key_raw = self.get_token_value(&tokens[*i]);
                    let key = self.interpolate_string(&key_raw);
                    *i += 1;
                    if *i < tokens.len() {
                        let path_raw = self.get_token_value(&tokens[*i]);
                        let path = self.interpolate_string(&path_raw);
                        if let Ok(mut assets) = self.assets.write() {
                            assets.insert(key, path);
                        }
                    }
                }
            }
            TokenType::Play => {
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::Sound {
                    *i += 1;
                    if *i < tokens.len() {
                        let path_raw = self.get_token_value(&tokens[*i]);
                        let path = self.interpolate_string(&path_raw);
                        if let Some(tx) = &self.sound_tx {
                            let _ = tx.send(crate::sound::SoundCommand::Play(path));
                        }
                    }
                }
            }
            TokenType::Beep => {
                if let Some(tx) = &self.sound_tx {
                    let _ = tx.send(crate::sound::SoundCommand::Beep);
                }
            }
            _ => {}
        }
        if *i > start_index {
            *i -= 1;
        }
    }

    pub fn handle_assignment(&mut self, i: &mut usize, tokens: &Vec<Token>, var_name: String) {
        if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::Assign {
            *i += 2;
            if *i >= tokens.len() {
                return;
            }
            let is_warp_call = match &tokens[*i].token_type {
                TokenType::Warp => true,
                TokenType::Identifier(func) if func == "w" || func == "warp" => true,
                _ => false,
            };
            if is_warp_call
                && *i + 1 < tokens.len()
                && tokens[*i + 1].token_type == TokenType::LParen
            {
                let val = self.handle_warp_constructor(i, tokens);
                self.set_variable(var_name, val);
                *i -= 1;
                return;
            }
            if tokens[*i].token_type == TokenType::Solve
                && *i + 1 < tokens.len()
                && tokens[*i + 1].token_type == TokenType::LBracket
            {
                let res = self.handle_solve_constructor(i, tokens);
                if self.last_bug_found {
                    return;
                }
                self.set_variable(var_name, res);
                *i -= 1;
                return;
            }
            let mut is_math = false;
            let mut j = *i;
            while j < tokens.len() && tokens[j].line == tokens[*i].line {
                match tokens[j].token_type {
                    TokenType::Plus
                    | TokenType::Minus
                    | TokenType::Star
                    | TokenType::Slash
                    | TokenType::Remainder
                    | TokenType::Sqrt
                    | TokenType::Sin
                    | TokenType::Cos
                    | TokenType::Tan
                    | TokenType::Abs
                    | TokenType::Log
                    | TokenType::Minimum
                    | TokenType::Maximum
                    | TokenType::PI
                    | TokenType::Euler => {
                        is_math = true;
                        break;
                    }
                    _ => {}
                }
                j += 1;
            }
            if is_math {
                let mut math_tokens = Vec::new();
                let start_line = tokens[*i].line;
                while *i < tokens.len() && tokens[*i].line == start_line {
                    if tokens[*i].token_type == TokenType::Ampersand {
                        break;
                    }
                    match tokens[*i].token_type {
                        TokenType::Print
                        | TokenType::Set
                        | TokenType::Start
                        | TokenType::Loop
                        | TokenType::If
                        | TokenType::Or
                        | TokenType::End => break,
                        _ => {}
                    }
                    math_tokens.push(tokens[*i].clone());
                    *i += 1;
                }
                *i -= 1;
                let res = self.evaluate_math_recursive(&math_tokens);
                if self.last_bug_found {
                    return;
                }
                let final_val = if res.fract() == 0.0 {
                    format!("{:.0}", res)
                } else {
                    res.to_string()
                };
                self.set_variable(var_name, final_val);
            } else {
                let value = self.get_complex_value(i, tokens);
                if var_name == "item" || var_name.starts_with("item_") {
                    self.set_variable(value.clone(), String::from("0"));
                } else if var_name.ends_with("_value") {
                    let real_name = var_name.trim_end_matches("_value").to_string();
                    self.set_variable(real_name, value);
                } else {
                    self.set_variable(var_name, value);
                }
                *i -= 1;
            }
        }
    }
}
