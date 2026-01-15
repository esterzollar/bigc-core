use super::Interpreter;
use crate::tokens::{Token, TokenType};
use serde_json::Value;

impl Interpreter {
    pub fn handle_list(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        let start_index = *i;
        // 1. list add "Item" @{ListVar}
        // 2. list remove "Item" @{ListVar}
        // 3. list cut @Index @{ListVar}

        *i += 1; // Skip "list"
        if *i >= tokens.len() {
            return;
        }

        let action = tokens[*i].token_type.clone();
        *i += 1;

        match action {
            TokenType::Folder => {
                // Syntax: list folder @"path" & set as list {Files}
                if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                    if !self.validate_at_strictness(*i, tokens) {
                        return;
                    }
                    *i += 1;

                    let path_raw = self.get_complex_value(i, tokens);
                    let path = self.interpolate_string(&path_raw);

                    self.last_bug_found = false;
                    let mut file_list = Vec::new();

                    match std::fs::read_dir(&path) {
                        Ok(entries) => {
                            for entry in entries.flatten() {
                                if let Ok(name) = entry.file_name().into_string() {
                                    file_list.push(name);
                                }
                            }
                            // Sort for consistency
                            file_list.sort();
                            *i -= 1;
                            self.handle_set_as_multiple(i, tokens, file_list);
                        }
                        Err(e) => {
                            self.last_bug_found = true;
                            self.last_bug_type = format!("List Folder Error: {}", e);
                            *i -= 1;
                            self.handle_set_as_multiple(i, tokens, vec![String::from("nothing")]);
                        }
                    }
                }
            }
            TokenType::Add => {
                let val = self.get_token_value(&tokens[*i]);
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::To {
                    *i += 1;
                }

                if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                    if !self.validate_at_strictness(*i, tokens) {
                        return;
                    }
                    *i += 1;
                    let list_var_name = self.extract_braced_name(i, tokens);
                    let list_json = self
                        .get_variable(&list_var_name)
                        .unwrap_or(String::from("[]"));

                    if let Ok(mut parsed) = serde_json::from_str::<Value>(&list_json) {
                        if let Some(arr) = parsed.as_array_mut() {
                            arr.push(Value::String(val));
                            self.set_variable(list_var_name, parsed.to_string());
                        }
                    }
                }
            }
            TokenType::Remove => {
                let val = self.get_token_value(&tokens[*i]);
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::From {
                    *i += 1;
                }

                if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                    if !self.validate_at_strictness(*i, tokens) {
                        return;
                    }
                    *i += 1;
                    let list_var_name = self.extract_braced_name(i, tokens);
                    let list_json = self
                        .get_variable(&list_var_name)
                        .unwrap_or(String::from("[]"));

                    if let Ok(mut parsed) = serde_json::from_str::<Value>(&list_json) {
                        if let Some(arr) = parsed.as_array_mut() {
                            arr.retain(|v| v.as_str() != Some(&val));
                            self.set_variable(list_var_name, parsed.to_string());
                        }
                    }
                }
            }
            TokenType::Cut => {
                // Syntax: list cut @1 @{List}
                // *i is at first @
                if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                    if !self.validate_at_strictness(*i, tokens) {
                        return;
                    }
                    *i += 1;
                    let index_str = self.get_token_value(&tokens[*i]);
                    let index_raw = index_str.parse::<f64>().unwrap_or(0.0) as usize;
                    let index = if index_raw > 0 { index_raw - 1 } else { 0 };
                    *i += 1;

                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        *i += 1;
                        let list_var_name = self.extract_braced_name(i, tokens);
                        let list_json = self
                            .get_variable(&list_var_name)
                            .unwrap_or(String::from("[]"));

                        if let Ok(mut parsed) = serde_json::from_str::<Value>(&list_json) {
                            if let Some(arr) = parsed.as_array_mut() {
                                if index < arr.len() {
                                    arr.remove(index);
                                }
                                self.set_variable(list_var_name, parsed.to_string());
                            }
                        }
                    }
                }
            }
            TokenType::Sort => {
                if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                    if !self.validate_at_strictness(*i, tokens) {
                        return;
                    }
                    *i += 1;
                    let list_var_name = self.extract_braced_name(i, tokens);
                    let list_json = self
                        .get_variable(&list_var_name)
                        .unwrap_or(String::from("[]"));

                    if let Ok(mut parsed) = serde_json::from_str::<Value>(&list_json) {
                        if let Some(arr) = parsed.as_array_mut() {
                            arr.sort_by(|a, b| {
                                let sa = match a {
                                    Value::String(s) => s.clone(),
                                    _ => a.to_string(),
                                };
                                let sb = match b {
                                    Value::String(s) => s.clone(),
                                    _ => b.to_string(),
                                };
                                sa.cmp(&sb)
                            });
                            self.set_variable(list_var_name, parsed.to_string());
                        }
                    }
                }
            }
            TokenType::Insert => {
                // list insert "Item" at @Index @{List}
                let val = self.get_token_value(&tokens[*i]);
                *i += 1;

                // Handle optional 'at' keyword
                if *i < tokens.len() && tokens[*i].token_type == TokenType::AtWord {
                    *i += 1;
                }

                if *i < tokens.len() && tokens[*i].token_type == TokenType::On {
                    *i += 1;
                }
                if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                    if !self.validate_at_strictness(*i, tokens) {
                        return;
                    }
                    *i += 1;
                    let index_str = self.get_token_value(&tokens[*i]);
                    let index_raw = index_str.parse::<f64>().unwrap_or(0.0) as usize;
                    let index = if index_raw > 0 { index_raw - 1 } else { 0 };
                    *i += 1;

                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        *i += 1;
                        let list_var_name = self.extract_braced_name(i, tokens);
                        let list_json = self
                            .get_variable(&list_var_name)
                            .unwrap_or(String::from("[]"));

                        if let Ok(mut parsed) = serde_json::from_str::<Value>(&list_json) {
                            if let Some(arr) = parsed.as_array_mut() {
                                let insert_pos = if index > arr.len() { arr.len() } else { index };
                                arr.insert(insert_pos, Value::String(val));
                                self.set_variable(list_var_name, parsed.to_string());
                            }
                        }
                    }
                }
            }
            _ => {
                *i -= 1;
            }
        }
        if *i > start_index {
            *i -= 1;
        }
    }

    pub fn handle_get_from(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax: get from {List} at 1 & set as {Item}
        *i += 1; // Skip "from"

        if *i < tokens.len() {
            // Check for optional braces
            if tokens[*i].token_type == TokenType::LBrace {
                *i += 1;
            }

            let list_var_name = if let TokenType::Identifier(s) = &tokens[*i].token_type {
                s.clone()
            } else {
                String::new()
            };

            if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::RBrace {
                *i += 1;
            }

            let list_json = self
                .get_variable(&list_var_name)
                .unwrap_or(String::from("[]"));

            *i += 1; // skip name or }
                     // *i is now at next token
            if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                if !self.validate_at_strictness(*i, tokens) {
                    return;
                }
                *i += 1;

                if *i < tokens.len() {
                    let index_str = self.get_token_value(&tokens[*i]);

                    // Smart parse: handle "1.0" by parsing as f64 first
                    let index_raw = index_str.parse::<f64>().unwrap_or(0.0) as usize;

                    // Human Logic: 1 -> 0
                    let index = if index_raw > 0 { index_raw - 1 } else { 0 };

                    if let Ok(parsed) = serde_json::from_str::<Value>(&list_json) {
                        if let Some(arr) = parsed.as_array() {
                            let result = if index < arr.len() {
                                match &arr[index] {
                                    Value::String(s) => s.clone(),
                                    Value::Number(n) => n.to_string(),
                                    _ => arr[index].to_string(),
                                }
                            } else {
                                String::from("nothing")
                            };
                            self.handle_set_as_multiple(i, tokens, vec![result]);
                        }
                    }
                }
            }
        }
    }

    pub fn handle_get_count(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax: get count of {List} & set as {Size}
        *i += 1; // Skip "count" identifier (handled in get.rs)

        if *i < tokens.len() && tokens[*i].token_type == TokenType::Of {
            *i += 1; // Skip "of"

            // Check for optional braces
            if tokens[*i].token_type == TokenType::LBrace {
                *i += 1;
            }

            let list_var_name = if let TokenType::Identifier(s) = &tokens[*i].token_type {
                s.clone()
            } else {
                String::new()
            };

            if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::RBrace {
                *i += 1;
            }

            let list_json = self
                .get_variable(&list_var_name)
                .unwrap_or(String::from("[]"));

            let count = if let Ok(parsed) = serde_json::from_str::<Value>(&list_json) {
                if let Some(arr) = parsed.as_array() {
                    arr.len()
                } else {
                    0
                }
            } else {
                0
            };

            self.handle_set_as_multiple(i, tokens, vec![count.to_string()]);
        }
    }
}
