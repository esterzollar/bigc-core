use super::Interpreter;
use crate::tokens::{Token, TokenType};
use std::fs::{self, OpenOptions};
use std::io::Write;

impl Interpreter {
    pub fn handle_books(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        let start_index = *i;
        let token_type = tokens[*i].token_type.clone();

        match token_type {
            TokenType::Open => {
                // Syntax: open "filename.txt" & set as {Content}
                let mut temp_i = *i + 1;
                let filename = self.get_complex_value(&mut temp_i, tokens);
                *i = temp_i - 1;

                if !filename.is_empty() {
                    self.last_bug_found = false;
                    let content = match self.resolve_file_as_string(&filename) {
                        Some(c) => c,
                        None => {
                            self.last_bug_found = true;
                            let err_msg = format!("File Error: Could not read '{}'", filename);
                            self.last_bug_type = err_msg.clone();
                            self.set_variable("BugType".to_string(), err_msg);
                            String::new()
                        }
                    };

                    // Reuse standard "set as" logic
                    self.handle_set_as_multiple(i, tokens, vec![content]);
                    return;
                }
            }
            TokenType::Write | TokenType::Add => {
                // Syntax: write "Content" @ "filename.txt"
                // Syntax: add "More Content" @ "filename.txt"
                let is_append = token_type == TokenType::Add;

                *i += 1;
                if *i < tokens.len() {
                    let content_raw = self.get_complex_value(i, tokens);
                    let content = self.interpolate_string(&content_raw);

                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        *i += 1;

                        if *i < tokens.len() {
                            let filename_raw = self.get_complex_value(i, tokens);
                            let filename = self.interpolate_string(&filename_raw);

                            self.last_bug_found = false;
                            let file_res = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(!is_append)
                                .append(is_append)
                                .open(&filename);

                            match file_res {
                                Ok(mut f) => {
                                    if let Err(e) = write!(f, "{}", content) {
                                        self.last_bug_found = true;
                                        self.last_bug_type = format!("Write Error: {}", e);
                                    }
                                }
                                Err(e) => {
                                    self.last_bug_found = true;
                                    self.last_bug_type = format!("Open Error: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            TokenType::Create => {
                // Syntax: create folder "Name" @"path"
                // Syntax: create file "Name" @"path"
                *i += 1; // Skip create
                let is_folder = *i < tokens.len() && tokens[*i].token_type == TokenType::Folder;
                let is_file = *i < tokens.len() && tokens[*i].token_type == TokenType::File;

                if is_folder || is_file {
                    *i += 1;
                    if *i < tokens.len() {
                        let name_raw = self.get_complex_value(i, tokens);
                        let name = self.interpolate_string(&name_raw);

                        if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                            if !self.validate_at_strictness(*i, tokens) {
                                return;
                            }
                            *i += 1;

                            let path_raw = self.get_complex_value(i, tokens);
                            let path = self.interpolate_string(&path_raw);

                            let full_path = std::path::Path::new(&path).join(&name);

                            self.last_bug_found = false;
                            if is_folder {
                                if let Err(e) = fs::create_dir_all(&full_path) {
                                    self.last_bug_found = true;
                                    self.last_bug_type = format!("Create Folder Error: {}", e);
                                }
                            } else {
                                // Create empty file
                                if let Err(e) = std::fs::File::create(&full_path) {
                                    self.last_bug_found = true;
                                    self.last_bug_type = format!("Create File Error: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            TokenType::Delete => {
                // Syntax: delete file @"path"
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::File {
                    *i += 1;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        *i += 1;
                        let path_raw = self.get_complex_value(i, tokens);
                        let path = self.interpolate_string(&path_raw);

                        self.last_bug_found = false;
                        if let Err(e) = fs::remove_file(&path) {
                            if let Err(_e2) = fs::remove_dir_all(&path) {
                                self.last_bug_found = true;
                                self.last_bug_type = format!("Delete Error: {}", e);
                            }
                        }
                    }
                }
            }
            TokenType::Copy => {
                // Syntax: copy file @"src" to @"dst"
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::File {
                    *i += 1;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        *i += 1;
                        let src_raw = self.get_complex_value(i, tokens);
                        let src = self.interpolate_string(&src_raw);

                        if *i < tokens.len() && tokens[*i].token_type == TokenType::To {
                            *i += 1;
                            if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                                if !self.validate_at_strictness(*i, tokens) {
                                    return;
                                }
                                *i += 1;
                                let dst_raw = self.get_complex_value(i, tokens);
                                let dst = self.interpolate_string(&dst_raw);

                                self.last_bug_found = false;
                                if let Err(e) = fs::copy(&src, &dst) {
                                    self.last_bug_found = true;
                                    self.last_bug_type = format!("Copy Error: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            TokenType::Move => {
                // Syntax: move file @"src" to @"dst"
                if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::File {
                    *i += 2;

                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        *i += 1;
                        let src_raw = self.get_complex_value(i, tokens);
                        let src = self.interpolate_string(&src_raw);

                        if *i < tokens.len() && tokens[*i].token_type == TokenType::To {
                            *i += 1;
                            if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                                if !self.validate_at_strictness(*i, tokens) {
                                    return;
                                }
                                *i += 1;
                                let dst_raw = self.get_complex_value(i, tokens);
                                let dst = self.interpolate_string(&dst_raw);

                                self.last_bug_found = false;
                                if let Err(e) = fs::rename(&src, &dst) {
                                    self.last_bug_found = true;
                                    self.last_bug_type = format!("Move Error: {}", e);
                                }
                            }
                        }
                    }
                } else {
                    return;
                }
            }
            _ => {}
        }
        if *i > start_index {
            *i -= 1;
        }
    }
}
