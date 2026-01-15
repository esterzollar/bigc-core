use super::Interpreter;
use crate::tokens::{Token, TokenType};
use serde_json::Value;

impl Interpreter {
    pub fn handle_map(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // 1. map set "Key" as "Val" @{MapVar}
        // 2. map get "Key" of {MapVar} & set as {Var}
        // 3. map check "Key" of {MapVar} & set as {Found}

        let start_index = *i;
        *i += 1; // Skip "map"
        if *i >= tokens.len() {
            return;
        }

        let action = tokens[*i].token_type.clone();

        match action {
            TokenType::Set => self.handle_map_set(i, tokens),
            TokenType::Get => self.handle_map_get(i, tokens),
            TokenType::Check => self.handle_map_check(i, tokens),
            TokenType::Remove => self.handle_map_remove(i, tokens),
            TokenType::Merge => self.handle_map_merge(i, tokens),
            _ => {}
        }

        // Ensure we don't double-skip if a handler already backed up
        if *i > start_index {
            *i -= 1;
        }
    }

    fn handle_map_merge(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "merge"

        // Get Source Map (handle braces)
        let source_var_name = self.extract_braced_name(i, tokens);
        let source_json = self
            .get_variable(&source_var_name)
            .unwrap_or(String::from("{}"));

        if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
            if !self.validate_at_strictness(*i, tokens) {
                return;
            }
            *i += 1;
            let target_var_name = self.extract_braced_name(i, tokens);
            let target_json = self
                .get_variable(&target_var_name)
                .unwrap_or(String::from("{}"));

            if let (Ok(mut t_parsed), Ok(s_parsed)) = (
                serde_json::from_str::<Value>(&target_json),
                serde_json::from_str::<Value>(&source_json),
            ) {
                if let (Some(t_obj), Some(s_obj)) = (t_parsed.as_object_mut(), s_parsed.as_object())
                {
                    for (k, v) in s_obj {
                        t_obj.insert(k.clone(), v.clone());
                    }
                    self.set_variable(target_var_name, t_parsed.to_string());
                }
            }
        }
    }

    fn handle_map_remove(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "remove"
        let key = self.get_token_value(&tokens[*i]);
        *i += 1;

        if *i < tokens.len() && tokens[*i].token_type == TokenType::From {
            *i += 1;
        }

        if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
            if !self.validate_at_strictness(*i, tokens) {
                return;
            }
            *i += 1;
            let map_var_name = self.extract_braced_name(i, tokens);
            let map_json = self
                .get_variable(&map_var_name)
                .unwrap_or(String::from("{}"));

            if let Ok(mut parsed) = serde_json::from_str::<Value>(&map_json) {
                if let Some(obj) = parsed.as_object_mut() {
                    obj.remove(&key);
                    self.set_variable(map_var_name, parsed.to_string());
                }
            }
        }
    }

    fn handle_map_set(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "set"
        let key = self.get_token_value(&tokens[*i]);
        *i += 1;

        if *i < tokens.len() && tokens[*i].token_type == TokenType::As {
            *i += 1;
            let val = self.get_token_value(&tokens[*i]);
            *i += 1;

            if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                if !self.validate_at_strictness(*i, tokens) {
                    return;
                }
                *i += 1;

                if *i < tokens.len() {
                    // Get Map variable name (handle braces)
                    let map_var_name = self.extract_braced_name(i, tokens);
                    let map_json = self
                        .get_variable(&map_var_name)
                        .unwrap_or(String::from("{}"));

                    if let Ok(mut parsed) = serde_json::from_str::<Value>(&map_json) {
                        if let Some(obj) = parsed.as_object_mut() {
                            obj.insert(key, Value::String(val));
                            self.set_variable(map_var_name, parsed.to_string());
                        }
                    }
                }
            }
        }
    }

    fn handle_map_get(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "get"
        let key = self.get_token_value(&tokens[*i]);
        *i += 1;

        if *i < tokens.len() && tokens[*i].token_type == TokenType::Of {
            *i += 1;
            let map_var_name = self.extract_braced_name(i, tokens);
            let map_json = self
                .get_variable(&map_var_name)
                .unwrap_or(String::from("{}"));

            if let Ok(parsed) = serde_json::from_str::<Value>(&map_json) {
                if let Some(obj) = parsed.as_object() {
                    let result = match obj.get(&key) {
                        Some(v) => match v {
                            Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        },
                        None => String::from("nothing"),
                    };
                    *i -= 1; // Back up so handle_set looks at next token (&)
                    self.handle_set_as_multiple(i, tokens, vec![result]);
                }
            }
        }
    }

    fn handle_map_check(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "check"
        let key = self.get_token_value(&tokens[*i]);
        *i += 1;

        if *i < tokens.len() && tokens[*i].token_type == TokenType::Of {
            *i += 1;
            let map_var_name = self.extract_braced_name(i, tokens);
            let map_json = self
                .get_variable(&map_var_name)
                .unwrap_or(String::from("{}"));

            let found = if let Ok(parsed) = serde_json::from_str::<Value>(&map_json) {
                parsed
                    .as_object()
                    .map(|obj| obj.contains_key(&key))
                    .unwrap_or(false)
            } else {
                false
            };

            *i -= 1; // Back up so handle_set looks at next token (&)
            self.handle_set_as_multiple(
                i,
                tokens,
                vec![if found {
                    "true".to_string()
                } else {
                    "false".to_string()
                }],
            );
        }
    }
}
