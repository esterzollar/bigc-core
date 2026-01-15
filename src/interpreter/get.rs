use super::Interpreter;
use crate::tokens::{Token, TokenType};
use std::time::Instant;

impl Interpreter {
    pub fn handle_get(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        let start_index = *i;
        let start_type = tokens[*i].token_type.clone();
        *i += 1;
        if *i >= tokens.len() {
            return;
        }

        // Special Check: Is this a specialized identifier (web, time, count)?
        // If start_type is Get, we look at the CURRENT token (tokens[*i])
        // If start_type is Identifier (legacy direct call), we use that.
        let mut special_cmd = String::new();

        if let TokenType::Identifier(ref s) = start_type {
            special_cmd = s.clone();
        } else if start_type == TokenType::Get {
            if let TokenType::Identifier(ref s) = tokens[*i].token_type {
                if s == "web" || s == "time" || s == "count" {
                    special_cmd = s.clone();
                    *i += 1; // Consume the sub-command (web/time/count)
                }
            }
        }

        if !special_cmd.is_empty()
            && (special_cmd == "web" || special_cmd == "time" || special_cmd == "count") {
                match special_cmd.as_str() {
                    "web" => {
                        if *i < tokens.len() {
                            let url = self.get_token_value(&tokens[*i]);
                            self.last_bug_found = false;
                            let resp_str = self.net.get(&url);
                            if resp_str.starts_with("BigNet Error") {
                                self.last_bug_found = true;
                                self.last_bug_type = resp_str.clone();
                                self.set_variable("BugType".to_string(), resp_str.clone());
                            }
                            self.handle_set_as_multiple(i, tokens, vec![resp_str]);
                        }
                    }
                    "time" => {
                        if *i < tokens.len() {
                            let arg = self.get_token_value(&tokens[*i]);
                            if arg == "unix" {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                self.handle_set_as_multiple(i, tokens, vec![now.to_string()]);
                            } else if arg == "tick" {
                                let ms = self.start_time.elapsed().as_millis();
                                self.handle_set_as_multiple(i, tokens, vec![ms.to_string()]);
                            } else if arg == "delta" {
                                let now = Instant::now();
                                let delta = now.duration_since(self.last_delta_tick).as_millis();
                                self.last_delta_tick = now;
                                self.handle_set_as_multiple(i, tokens, vec![delta.to_string()]);
                            }
                        }
                    }
                    "count" => {
                        self.handle_get_count(i, tokens);
                    }
                    _ => {}
                }
                // Check if we need to backup (if handle_set_as_multiple advanced too far? No, it handles it)
                // Actually, handle_set_as_multiple advances past 'set as {Var}'.
                // We should return here.
                return;
            }

        match start_type {
            TokenType::Command => {
                let args: Vec<String> = std::env::args().skip(2).collect();
                self.handle_set_as_multiple(i, tokens, args);
            }
            TokenType::Get => {
                let mut as_pos = None;
                let mut j = *i;
                let line = tokens[*i].line;
                while j < tokens.len() && tokens[j].line == line {
                    if tokens[j].token_type == TokenType::Ampersand {
                        break;
                    }
                    if tokens[j].token_type == TokenType::As {
                        as_pos = Some(j);
                        break;
                    }
                    j += 1;
                }

                if let Some(pos) = as_pos {
                    if tokens[*i].token_type != TokenType::Len {
                        let mut inputs = Vec::new();
                        let mut k = *i;
                        while k < pos {
                            let raw = self.get_token_value(&tokens[k]);
                            inputs.push(self.interpolate_string(&raw));
                            k += 1;
                        }

                        let kind = if pos + 1 < tokens.len() {
                            self.get_token_value(&tokens[pos + 1])
                        } else {
                            String::new()
                        };
                        let val = if !inputs.is_empty() {
                            inputs[0].clone()
                        } else {
                            String::new()
                        };

                        let res = match kind.to_lowercase().as_str() {
                            "email" => {
                                let re = regex::Regex::new(
                                    r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
                                )
                                .unwrap();
                                re.is_match(&val).to_string()
                            }
                            "number" => val.parse::<f64>().is_ok().to_string(),
                            "url" => (val.starts_with("http://") || val.starts_with("https://"))
                                .to_string(),
                            "alphanumeric" => val.chars().all(|c| c.is_alphanumeric()).to_string(),
                            "clean" => val.trim().to_string(),
                            "bigcap" => val.to_uppercase(),
                            "lower" => val.to_lowercase(),
                            "floor" => val.parse::<f64>().unwrap_or(0.0).floor().to_string(),
                            "ceil" => val.parse::<f64>().unwrap_or(0.0).ceil().to_string(),
                            "round" => val.parse::<f64>().unwrap_or(0.0).round().to_string(),
                            "abs" | "positive" => {
                                let n = if val == "-" && inputs.len() > 1 {
                                    inputs[1].parse::<f64>().unwrap_or(0.0)
                                } else {
                                    val.parse::<f64>().unwrap_or(0.0)
                                };
                                n.abs().to_string()
                            }
                            "smaller" => {
                                let n1 = val.parse::<f64>().unwrap_or(0.0);
                                let n2 = if inputs.len() > 1 {
                                    inputs[1].parse::<f64>().unwrap_or(0.0)
                                } else {
                                    0.0
                                };
                                if n1 < n2 {
                                    n1.to_string()
                                } else {
                                    n2.to_string()
                                }
                            }
                            "bigger" => {
                                let n1 = val.parse::<f64>().unwrap_or(0.0);
                                let n2 = if inputs.len() > 1 {
                                    inputs[1].parse::<f64>().unwrap_or(0.0)
                                } else {
                                    0.0
                                };
                                if n1 > n2 {
                                    n1.to_string()
                                } else {
                                    n2.to_string()
                                }
                            }
                            "between" => {
                                let v = val.parse::<f64>().unwrap_or(0.0);
                                let min = if inputs.len() > 1 {
                                    inputs[1].parse::<f64>().unwrap_or(0.0)
                                } else {
                                    0.0
                                };
                                let max = if inputs.len() > 2 {
                                    inputs[2].parse::<f64>().unwrap_or(0.0)
                                } else {
                                    0.0
                                };
                                v.clamp(min, max).to_string()
                            }
                            "len" | "length" | "count" => {
                                if val.starts_with('[') && val.ends_with(']') {
                                    let list = self.parse_json_list(&val);
                                    list.len().to_string()
                                } else {
                                    val.len().to_string()
                                }
                            }
                            _ => "false".to_string(),
                        };

                        *i = pos + 1;
                        self.handle_set_as_multiple(i, tokens, vec![res]);
                        *i += 1;
                        if *i > start_index {
                            *i -= 1;
                        }
                        return;
                    }
                }

                match &tokens[*i].token_type {
                    TokenType::Len => {
                        let mut temp_i = *i;
                        let val = self.get_complex_value(&mut temp_i, tokens);
                        *i = temp_i - 1;
                        self.handle_set_as_multiple(i, tokens, vec![val]);
                    }
                    TokenType::From => {
                        self.handle_get_from(i, tokens);
                    }
                    TokenType::String(_)
                    | TokenType::Dollar
                    | TokenType::Identifier(_)
                    | TokenType::Text
                    | TokenType::Image
                    | TokenType::Button
                    | TokenType::Font
                    | TokenType::Value => {
                        let mut items = Vec::new();
                        while *i < tokens.len() {
                            if tokens[*i].token_type == TokenType::Ampersand {
                                break;
                            }
                            let mut current_val = self.get_complex_value(i, tokens);
                            current_val = self.interpolate_string(&current_val);
                            if *i < tokens.len() && tokens[*i].token_type == TokenType::Replace {
                                *i += 1;
                                let old_val_raw = self.get_complex_value(i, tokens);
                                let old_val = self.interpolate_string(&old_val_raw);
                                if *i < tokens.len() && tokens[*i].token_type == TokenType::With {
                                    *i += 1;
                                    let new_val_raw = self.get_complex_value(i, tokens);
                                    let new_val = self.interpolate_string(&new_val_raw);
                                    current_val = current_val.replace(&old_val, &new_val);
                                }
                            }
                            items.push(current_val);
                            if *i < tokens.len() {
                                match &tokens[*i].token_type {
                                    TokenType::String(_)
                                    | TokenType::Dollar
                                    | TokenType::Identifier(_) => {}
                                    _ => break,
                                }
                            }
                        }
                        *i -= 1;
                        self.handle_set_as_multiple(i, tokens, items);
                    }
                    TokenType::Warp => {
                        *i += 1;
                        if *i < tokens.len() {
                            let raw_text = self.get_token_value(&tokens[*i]);
                            let warped = self.interpolate_string(&raw_text);
                            self.handle_set_as_multiple(i, tokens, vec![warped]);
                        }
                    }
                    TokenType::Luck => {
                        *i += 1;
                        match &tokens[*i].token_type {
                            TokenType::Random => {
                                *i += 1;
                                let min_val_raw = self.get_token_value(&tokens[*i]);
                                let min_val = self.interpolate_string(&min_val_raw);
                                *i += 1;
                                let max_val_raw = self.get_token_value(&tokens[*i]);
                                let max_val = self.interpolate_string(&max_val_raw);
                                let min = min_val.parse::<i32>().unwrap_or(0);
                                let max = max_val.parse::<i32>().unwrap_or(100);
                                let res = self.luck.get_random_num(min, max);
                                self.handle_set_as_multiple(i, tokens, vec![res]);
                            }
                            TokenType::Identifier(kind) => {
                                if kind == "name" {
                                    let res = vec![self.luck.get_first(), self.luck.get_last()];
                                    self.handle_set_as_multiple(i, tokens, res);
                                } else if kind == "first" {
                                    self.handle_set_as_multiple(
                                        i,
                                        tokens,
                                        vec![self.luck.get_first()],
                                    );
                                } else if kind == "last" {
                                    self.handle_set_as_multiple(
                                        i,
                                        tokens,
                                        vec![self.luck.get_last()],
                                    );
                                } else if kind == "zip" {
                                    self.handle_set_as_multiple(
                                        i,
                                        tokens,
                                        vec![self.luck.get_zip()],
                                    );
                                } else if kind == "street" {
                                    self.handle_set_as_multiple(
                                        i,
                                        tokens,
                                        vec![self.luck.get_street()],
                                    );
                                } else if kind == "ua" {
                                    self.handle_set_as_multiple(
                                        i,
                                        tokens,
                                        vec![self.luck.get_user_agent()],
                                    );
                                } else if kind == "random" {
                                    *i += 1;
                                    let min_val_raw = self.get_token_value(&tokens[*i]);
                                    let min_val = self.interpolate_string(&min_val_raw);
                                    *i += 1;
                                    let max_val_raw = self.get_token_value(&tokens[*i]);
                                    let max_val = self.interpolate_string(&max_val_raw);
                                    let min = min_val.parse::<i32>().unwrap_or(0);
                                    let max = max_val.parse::<i32>().unwrap_or(100);
                                    let res = self.luck.get_random_num(min, max);
                                    self.handle_set_as_multiple(i, tokens, vec![res]);
                                } else if kind == "number" {
                                    // Legacy: luck number 1 to 100
                                    *i += 1;
                                    let min_val_raw = self.get_token_value(&tokens[*i]);
                                    let min_val = self.interpolate_string(&min_val_raw);
                                    *i += 1;
                                    if tokens[*i].token_type == TokenType::To {
                                        *i += 1;
                                    }
                                    let max_val_raw = self.get_token_value(&tokens[*i]);
                                    let max_val = self.interpolate_string(&max_val_raw);
                                    let min = min_val.parse::<i32>().unwrap_or(0);
                                    let max = max_val.parse::<i32>().unwrap_or(100);
                                    let res = self.luck.get_random_num(min, max);
                                    self.handle_set_as_multiple(i, tokens, vec![res]);
                                }
                            }
                            TokenType::Email => {
                                let res = vec![self
                                    .luck
                                    .get_email(&self.luck.get_first(), &self.luck.get_last())];
                                self.handle_set_as_multiple(i, tokens, res);
                            }
                            TokenType::UUID => {
                                self.handle_set_as_multiple(i, tokens, vec![self.luck.get_uuid()]);
                            }
                            _ => {}
                        }
                    }
                    TokenType::Markdown => {
                        let mut temp_i = *i + 1;
                        let md_text = self.get_complex_value(&mut temp_i, tokens);
                        *i = temp_i - 1;
                        use pulldown_cmark::{html, Options, Parser};
                        let mut options = Options::empty();
                        options.insert(Options::ENABLE_TABLES);
                        options.insert(Options::ENABLE_STRIKETHROUGH);
                        let parser = Parser::new_ext(&md_text, options);
                        let mut html_output = String::new();
                        html::push_html(&mut html_output, parser);
                        self.handle_set_as_multiple(i, tokens, vec![html_output]);
                    }
                    TokenType::Setting => {
                        *i += 1;
                        let key = self.get_token_value(&tokens[*i]);
                        let val = std::env::var(&key).unwrap_or_default();
                        self.handle_set_as_multiple(i, tokens, vec![val]);
                    }
                    TokenType::LBrace => {
                        if *i + 1 < tokens.len() {
                            if let TokenType::Identifier(map_name) = &tokens[*i + 1].token_type {
                                let val = self.get_variable(map_name).unwrap_or(String::from("{}"));
                                *i += 2;
                                if *i < tokens.len() && tokens[*i].token_type == TokenType::RBrace {
                                    self.handle_set_as_multiple(i, tokens, vec![val]);
                                }
                            }
                        }
                    }
                    TokenType::Post => {
                        *i += 1;
                        let url = self.get_token_value(&tokens[*i]);
                        *i += 1;
                        if *i < tokens.len() && tokens[*i].token_type == TokenType::With {
                            *i += 1;
                        }
                        let raw_data = self.get_token_value(&tokens[*i]);
                        let data = self.interpolate_string(&raw_data);
                        self.last_bug_found = false;
                        let resp_str = self.net.post(&url, &data);
                        if resp_str.starts_with("BigNet Error") {
                            self.last_bug_found = true;
                            self.last_bug_type = resp_str.clone();
                            self.set_variable("BugType".to_string(), resp_str.clone());
                        }
                        self.handle_set_as_multiple(i, tokens, vec![resp_str]);
                    }
                    _ => {}
                }
            }
            TokenType::Look => {
                if tokens[*i].token_type == TokenType::For {
                    *i += 1;
                }
                let mut is_json = false;
                while *i < tokens.len() {
                    if tokens[*i].token_type == TokenType::Json {
                        is_json = true;
                        *i += 1;
                    } else if tokens[*i].token_type == TokenType::In {
                        *i += 1;
                    } else {
                        break;
                    }
                }
                let mut selectors = Vec::new();
                while *i < tokens.len() && tokens[*i].token_type != TokenType::At {
                    selectors.push(self.get_token_value(&tokens[*i]));
                    *i += 1;
                }
                if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                    if !self.validate_at_strictness(*i, tokens) {
                        return;
                    }
                    *i += 1;
                    let mut source =
                        if *i < tokens.len() && tokens[*i].token_type == TokenType::LBrace {
                            *i += 1;
                            let s = self.get_token_value(&tokens[*i]);
                            *i += 1;
                            s
                        } else {
                            self.get_token_value(&tokens[*i])
                        };
                    if source.ends_with(".biew") {
                        if let Ok(content) = std::fs::read_to_string(&source) {
                            source = super::biew::Biew::transpile_biew(&content);
                        }
                    }
                    let mut results = Vec::new();
                    for selector in selectors {
                        let res = if selector == "all" {
                            source.clone()
                        } else if is_json {
                            self.net.look_at_json(&selector, &source)
                        } else {
                            self.net.look_for(&selector, &source)
                        };
                        results.push(res);
                    }
                    self.handle_set_as_multiple(i, tokens, results);
                }
            }
            _ => {}
        }

        if *i > start_index {
            *i -= 1;
        }
    }
}
