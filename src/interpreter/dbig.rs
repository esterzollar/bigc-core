use super::Interpreter;
use crate::tokens::{Token, TokenType};
use serde_json::Value;
use std::fs::{self};
use std::io::Write;

impl Interpreter {
    pub fn handle_dbig(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax:
        // 1. dbig get "Key" @"file"
        // 2. dbig set "Key" as "Val" @"file"
        // 3. dbig remove "Key" @"file"
        // 4. dbig check ... (Existence, List Search, Scanner)

        *i += 1; // Skip "dbig"

        if *i >= tokens.len() {
            return;
        }

        let action = tokens[*i].token_type.clone();

        match action {
            TokenType::Get => {
                self.handle_dbig_get(i, tokens);
            }
            TokenType::Set => {
                self.handle_dbig_set(i, tokens);
            }
            TokenType::Remove => {
                self.handle_dbig_remove(i, tokens);
            }
            TokenType::Check => {
                self.handle_dbig_check(i, tokens);
            }
            _ => {}
        }
    }

    fn handle_dbig_get(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "get"
        let key = self.get_complex_value(i, tokens);

        if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
            if !self.validate_at_strictness(*i, tokens) {
                return;
            }
            *i += 1;

            if *i < tokens.len() {
                let filename_raw = self.get_token_value(&tokens[*i]);
                let filename = self.interpolate_string(&filename_raw);

                if !filename.ends_with(".dbig") {
                    println!("Big Error: DBB only works with .dbig files!");
                    return;
                }

                let content = fs::read_to_string(&filename).unwrap_or_default();
                let found_values = self.parse_dbig_content(&content, &key);
                self.handle_set_as_multiple(i, tokens, found_values);
            }
        }
    }

    fn handle_dbig_set(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "set"
        let key_raw = self.get_complex_value(i, tokens);
        let key = self.interpolate_string(&key_raw);

        if *i < tokens.len() && tokens[*i].token_type == TokenType::As {
            *i += 1;
            let mut values_to_write = Vec::new();
            let mut is_list_explicit = false;

            // Check for optional braces/list
            if tokens[*i].token_type == TokenType::LBrace {
                *i += 1;
            }
            if tokens[*i].token_type == TokenType::List {
                is_list_explicit = true;
                *i += 1;
            }

            let val_raw = self.get_complex_value(i, tokens);
            let val_interp = self.interpolate_string(&val_raw);

            if tokens[*i].token_type == TokenType::RBrace {
                *i += 1;
            }

            if is_list_explicit || val_interp.starts_with('[') {
                if let Ok(parsed) = serde_json::from_str::<Value>(&val_interp) {
                    if let Some(arr) = parsed.as_array() {
                        for item in arr {
                            match item {
                                Value::String(s) => values_to_write.push(s.clone()),
                                Value::Number(n) => values_to_write.push(n.to_string()),
                                _ => values_to_write.push(item.to_string()),
                            }
                        }
                    } else {
                        values_to_write.push(val_interp);
                    }
                } else {
                    values_to_write.push(val_interp);
                }
            } else {
                values_to_write.push(val_interp);
            }

            if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                if !self.validate_at_strictness(*i, tokens) {
                    return;
                }
                *i += 1;

                if *i < tokens.len() {
                    let filename_raw = self.get_token_value(&tokens[*i]);
                    let filename = self.interpolate_string(&filename_raw);

                    if !filename.ends_with(".dbig") {
                        println!("Big Error: DBB only works with .dbig files!");
                        return;
                    }

                    // --- ATOMIC SPIN-LOCK START ---
                    let lock_path = format!("{}.lock", filename);
                    let mut attempts = 0;
                    while attempts < 100 {
                        match std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(&lock_path)
                        {
                            Ok(_) => break, // Lock acquired atomically
                            Err(_) => {
                                std::thread::sleep(std::time::Duration::from_millis(10));
                                attempts += 1;
                            }
                        }
                    }
                    // --- LOCK ACQUIRED ---

                    let content = fs::read_to_string(&filename).unwrap_or_default();
                    let new_content = self.update_dbig_content(&content, &key, values_to_write);
                    let _ = fs::write(&filename, new_content);

                    // --- RELEASE LOCK ---
                    let _ = std::fs::remove_file(&lock_path);
                }
            }
        }
    }

    fn handle_dbig_remove(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // Skip "remove"
        let key = self.get_complex_value(i, tokens);

        if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
            if !self.validate_at_strictness(*i, tokens) {
                return;
            }
            *i += 1;

            if *i < tokens.len() {
                let filename_raw = self.get_token_value(&tokens[*i]);
                let filename = self.interpolate_string(&filename_raw);

                if !filename.ends_with(".dbig") {
                    println!("Big Error: DBB only works with .dbig files!");
                    return;
                }

                // --- ATOMIC SPIN-LOCK START ---
                let lock_path = format!("{}.lock", filename);
                let mut attempts = 0;
                while attempts < 100 {
                    match std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&lock_path)
                    {
                        Ok(_) => break,
                        Err(_) => {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            attempts += 1;
                        }
                    }
                }
                // --- LOCK ACQUIRED ---

                let content = fs::read_to_string(&filename).unwrap_or_default();
                let new_content = self.remove_dbig_block(&content, &key);
                let _ = fs::write(&filename, new_content);

                // --- RELEASE LOCK ---
                let _ = std::fs::remove_file(&lock_path);
            }
        }
    }

    fn handle_dbig_check(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Mode 1: dbig check "Key" @"file.dbig" (Existence)
        // Mode 2: dbig check "Item" of "Key" @"file.dbig" (List Search)
        // Mode 3: dbig check keys value < 10 @"file.dbig" (Scanner)

        *i += 1; // Skip "check"

        if *i >= tokens.len() {
            return;
        }

        if tokens[*i].token_type == TokenType::Keys {
            // Mode 3: Scanner
            self.handle_dbig_scan(i, tokens);
            return;
        }

        let target_item_or_key = self.get_complex_value(i, tokens);

        if *i < tokens.len() && tokens[*i].token_type == TokenType::Of {
            // Mode 2: List Search
            *i += 1; // Skip "of"
            let target_list_key = self.get_token_value(&tokens[*i]);
            *i += 1;

            if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                if !self.validate_at_strictness(*i, tokens) {
                    return;
                }
                *i += 1;

                if *i < tokens.len() {
                    let filename_raw = self.get_token_value(&tokens[*i]);
                    let filename = self.interpolate_string(&filename_raw);
                    if !filename.ends_with(".dbig") {
                        return;
                    }
                    let content = fs::read_to_string(&filename).unwrap_or_default();
                    let found =
                        self.check_item_in_list(&content, &target_list_key, &target_item_or_key);
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
        } else if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
            // Mode 1: Existence
            if !self.validate_at_strictness(*i, tokens) {
                return;
            }
            *i += 1;

            if *i < tokens.len() {
                let filename_raw = self.get_token_value(&tokens[*i]);
                let filename = self.interpolate_string(&filename_raw);
                if !filename.ends_with(".dbig") {
                    return;
                }
                let content = fs::read_to_string(&filename).unwrap_or_default();
                let found = self.check_dbig_key(&content, &target_item_or_key);
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

    fn handle_dbig_scan(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax: dbig check keys value < 10 @"file.dbig"
        *i += 1; // Skip "keys"

        if *i < tokens.len() && tokens[*i].token_type == TokenType::Value {
            *i += 1; // Skip "value"

            // Get Operator
            let operator = tokens[*i].token_type.clone();
            *i += 1;

            // Get Target Value
            let target_val_str = self.get_token_value(&tokens[*i]);
            let target_num = target_val_str.parse::<f64>().unwrap_or(0.0);
            *i += 1;

            if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                if !self.validate_at_strictness(*i, tokens) {
                    return;
                }
                *i += 1;

                if *i < tokens.len() {
                    let filename_raw = self.get_token_value(&tokens[*i]);
                    let filename = self.interpolate_string(&filename_raw);
                    if !filename.ends_with(".dbig") {
                        return;
                    }
                    let content = fs::read_to_string(&filename).unwrap_or_default();

                    let matches =
                        self.scan_dbig_values(&content, &operator, target_num, &target_val_str);
                    self.handle_set_as_multiple(i, tokens, matches);
                }
            }
        }
    }

    // --- PARSING LOGIC ---

    fn parse_dbig_content(&self, content: &str, target_key: &str) -> Vec<String> {
        let mut results = Vec::new();
        let mut inside_target = false;

        for line in content.lines() {
            let trim = line.trim();
            if trim.is_empty() {
                continue;
            }

            if trim.starts_with('[') && trim.ends_with(']') {
                let current_key = &trim[1..trim.len() - 1];
                inside_target = current_key == target_key;
            } else if inside_target
                && trim.starts_with("- ") && trim.ends_with(" |") {
                    let val = &trim[2..trim.len() - 2];
                    results.push(val.to_string());
                }
        }
        results
    }

    fn update_dbig_content(
        &self,
        content: &str,
        target_key: &str,
        new_values: Vec<String>,
    ) -> String {
        let mut new_lines = Vec::new();
        let mut skipping = false;

        for line in content.lines() {
            let trim = line.trim();

            if trim.starts_with('[') && trim.ends_with(']') {
                let current_key = &trim[1..trim.len() - 1];
                skipping = current_key == target_key;
            }

            if !skipping {
                new_lines.push(line.to_string());
            }
        }

        new_lines.push(String::new());
        new_lines.push(format!("[{}]", target_key));
        for val in new_values {
            new_lines.push(format!("- {} |", val));
        }
        new_lines.push(String::new());

        new_lines.join("\n")
    }

    fn remove_dbig_block(&self, content: &str, target_key: &str) -> String {
        let mut new_lines = Vec::new();
        let mut skipping = false;

        for line in content.lines() {
            let trim = line.trim();

            if trim.starts_with('[') && trim.ends_with(']') {
                let current_key = &trim[1..trim.len() - 1];
                skipping = current_key == target_key;
            }

            if !skipping {
                new_lines.push(line.to_string());
            }
        }
        new_lines.join("\n")
    }

    fn check_dbig_key(&self, content: &str, target_key: &str) -> bool {
        for line in content.lines() {
            let trim = line.trim();
            if trim.starts_with('[') && trim.ends_with(']') {
                let current_key = &trim[1..trim.len() - 1];
                if current_key == target_key {
                    return true;
                }
            }
        }
        false
    }

    fn check_item_in_list(&self, content: &str, target_key: &str, target_item: &str) -> bool {
        let mut inside_target = false;
        for line in content.lines() {
            let trim = line.trim();
            if trim.is_empty() {
                continue;
            }

            if trim.starts_with('[') && trim.ends_with(']') {
                let current_key = &trim[1..trim.len() - 1];
                inside_target = current_key == target_key;
            } else if inside_target
                && trim.starts_with("- ") && trim.ends_with(" |") {
                    let val = &trim[2..trim.len() - 2];
                    if val == target_item {
                        return true;
                    }
                }
        }
        false
    }

    fn scan_dbig_values(
        &self,
        content: &str,
        op: &TokenType,
        target_num: f64,
        target_str: &str,
    ) -> Vec<String> {
        let mut matches = Vec::new();
        let mut current_key = String::new();

        for line in content.lines() {
            let trim = line.trim();
            if trim.is_empty() {
                continue;
            }

            if trim.starts_with('[') && trim.ends_with(']') {
                current_key = trim[1..trim.len() - 1].to_string();
            } else if !current_key.is_empty()
                && trim.starts_with("- ") && trim.ends_with(" |") {
                    let val = &trim[2..trim.len() - 2];

                    let is_match = if let Ok(val_num) = val.parse::<f64>() {
                        match op {
                            TokenType::Greater => val_num > target_num,
                            TokenType::Less => val_num < target_num,
                            TokenType::Assign => (val_num - target_num).abs() < 0.0001,
                            _ => false,
                        }
                    } else {
                        // String comparison
                        match op {
                            TokenType::Assign => val == target_str,
                            _ => false,
                        }
                    };

                    if is_match {
                        matches.push(current_key.clone());
                    }
                }
        }
        // Remove duplicates if a key has multiple matching values?
        // Yes, scanning generally returns Keys.
        matches.dedup();
        matches
    }
}
