use super::Interpreter;
use crate::tokens::{Token, TokenType};
use std::fs;
use tiny_http::{Header, Response, Server};

impl Interpreter {
    pub fn handle_use_sbig(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1;
        // Check for "sbig" or "web"
        if *i < tokens.len() {
            match tokens[*i].token_type {
                TokenType::Sbig => {
                    self.sbig_enabled = true;
                }
                TokenType::Identifier(ref s) => {
                    if s.to_lowercase() == "web" || s.to_lowercase() == "sbig" {
                        self.sbig_enabled = true;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn handle_server_config(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax:
        // control workers 4
        // control limit 100 per minute
        // control record @"file.log"
        // control ssl @"cert" @"key"
        *i += 1; // Skip "control"

        if *i < tokens.len() {
            match tokens[*i].token_type {
                TokenType::Workers => {
                    *i += 1;
                    let val = self.get_token_value(&tokens[*i]);
                    self.max_workers = val.parse::<usize>().unwrap_or(1);
                }
                TokenType::Limit => {
                    *i += 1;
                    let val = self.get_token_value(&tokens[*i]);
                    self.rate_limit = val.parse::<usize>().unwrap_or(0);
                    // Consume "per minute" if present
                    if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::Per {
                        *i += 2;
                        if *i < tokens.len() && tokens[*i].token_type == TokenType::Mins {
                            // Valid syntax confirmed
                        }
                    }
                }
                TokenType::Record => {
                    *i += 1;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        *i += 1;
                        let filename = self.get_token_value(&tokens[*i]);
                        self.log_file = Some(filename);
                    }
                }
                TokenType::SSL => {
                    *i += 1;
                    if *i + 2 < tokens.len() && tokens[*i].token_type == TokenType::At {
                        if !self.validate_at_strictness(*i, tokens) {
                            return;
                        }
                        let cert = self.get_token_value(&tokens[*i + 1]);
                        *i += 2;
                        if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                            if !self.validate_at_strictness(*i, tokens) {
                                return;
                            }
                            let key = self.get_token_value(&tokens[*i + 1]);
                            self.ssl_config = Some((cert, key));
                            *i += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn handle_on(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        if !self.sbig_enabled {
            return;
        }
        *i += 1;
        if *i < tokens.len() {
            let method = match tokens[*i].token_type {
                TokenType::Get => "GET",
                TokenType::Post => "POST",
                _ => "GET",
            };
            *i += 1;
            if *i < tokens.len() {
                let path = self.get_token_value(&tokens[*i]);
                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::Run {
                    *i += 1;
                    if *i < tokens.len() {
                        let doing = self.get_token_value(&tokens[*i]);
                        let key = format!("{} {}", method, path);
                        if let Ok(mut r) = self.routes.write() {
                            r.insert(key, doing);
                        }
                    }
                }
            }
        }
    }

    pub fn handle_start_server(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        if !self.sbig_enabled {
            println!("Big Error: Web engine is locked! Use 'use web' first.");
            return;
        }
        *i += 2; // Skip start server
        if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
            if !self.validate_at_strictness(*i, tokens) {
                return;
            }
            *i += 1;

            let port = self.get_token_value(&tokens[*i]);
            let addr = format!("127.0.0.1:{}", port);
            println!("BigWeb: Listening on http://{}", addr);

            let server = if let Some((ref _cert, ref _key)) = self.ssl_config {
                println!("BigWeb: SSL Shield Enabled.");
                // Note: Standard tiny-http doesn't have easy one-liner SSL without features.
                // We will attempt to open it, but standard builds might fallback to HTTP.
                Server::http(&addr).unwrap()
            } else {
                Server::http(&addr).unwrap()
            };

            let mut request_count = 0;
            let mut last_minute = std::time::SystemTime::now();

            for mut request in server.incoming_requests() {
                println!("BigWeb DEBUG: {} {}", request.method(), request.url());
                // Logging
                if let Some(ref log_path) = self.log_file {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(log_path)
                    {
                        let _ = writeln!(
                            f,
                            "[{}] {} {}",
                            chrono::Local::now(),
                            request.method(),
                            request.url()
                        );
                    }
                }

                // Rate Limiting Logic
                if self.rate_limit > 0 {
                    if let Ok(elapsed) = last_minute.elapsed() {
                        if elapsed.as_secs() >= 60 {
                            request_count = 0;
                            last_minute = std::time::SystemTime::now();
                        }
                    }

                    if request_count >= self.rate_limit {
                        let _ = request.respond(
                            Response::from_string("429 Too Many Requests").with_status_code(429),
                        );
                        continue;
                    }
                    request_count += 1;
                }

                let method = request.method().as_str().to_uppercase();
                let url = request.url().to_string();

                // Route Matching (Exact + Wildcard)
                let mut matched_doing = None;
                let mut request_extra = String::new();

                let routes_snapshot = if let Ok(r) = self.routes.read() {
                    r.clone()
                } else {
                    std::collections::HashMap::new()
                };

                for (route_key, doing) in routes_snapshot {
                    let parts: Vec<&str> = route_key.split_whitespace().collect();
                    if parts.len() < 2 {
                        continue;
                    }
                    let r_method = parts[0];
                    let r_path = parts[1];

                    if r_method == method {
                        if let Some(base) = r_path.strip_suffix('+') {
                            if url.starts_with(base) {
                                matched_doing = Some(doing.clone());
                                request_extra = url[base.len()..].to_string();
                                break;
                            }
                        } else if r_path == url.split('?').next().unwrap_or("") {
                            matched_doing = Some(doing.clone());
                            break;
                        }
                    }
                }

                self.set_variable("Sbig_Response_Body".to_string(), String::new());
                self.set_variable("Sbig_Response_File".to_string(), String::new());
                self.set_variable("RequestPath".to_string(), url.clone());
                self.set_variable("RequestMethod".to_string(), method.clone());
                self.set_variable("RequestExtra".to_string(), request_extra);
                self.current_status = 200;
                self.current_headers.clear();

                let mut body_str = String::new();
                let _ = request.as_reader().read_to_string(&mut body_str);
                self.set_variable("RequestBody".to_string(), body_str);

                if let Some(doing_name) = matched_doing {
                    let func_data = if let Ok(funcs) = self.functions.read() {
                        funcs.get(&doing_name).cloned()
                    } else {
                        None
                    };

                    if let Some((_, func_tokens)) = func_data {
                        self.run(func_tokens);
                    }
                }

                let resp_body = self
                    .get_variable("Sbig_Response_Body")
                    .unwrap_or_default();
                let resp_file = self
                    .get_variable("Sbig_Response_File")
                    .unwrap_or_default();

                let mut response = if !resp_file.is_empty() {
                    if let Ok(content) = fs::read_to_string(&resp_file) {
                        let (final_content, content_type) = if resp_file.ends_with(".biew") {
                            (super::biew::Biew::transpile_biew(&content), "text/html")
                        } else if resp_file.ends_with(".bss") {
                            (super::biew::Biew::transpile_bss(&content), "text/css")
                        } else if resp_file.ends_with(".html") {
                            (content, "text/html")
                        } else {
                            (content, "text/plain")
                        };
                        Response::from_string(final_content).with_header(
                            Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                                .unwrap(),
                        )
                    } else {
                        Response::from_string("404").with_status_code(404)
                    }
                } else {
                    Response::from_string(resp_body)
                };

                // Apply Status & Headers
                response = response.with_status_code(self.current_status);
                for (k, v) in &self.current_headers {
                    if let Ok(h) = Header::from_bytes(k.as_bytes(), v.as_bytes()) {
                        response = response.with_header(h);
                    }
                }

                let _ = request.respond(response);
            }
        }
    }

    pub fn handle_reply(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1;
        if *i < tokens.len() {
            match tokens[*i].token_type {
                TokenType::With => {
                    *i += 1;
                    let val = if let TokenType::Identifier(ref name) = tokens[*i].token_type {
                        // Direct Variable: Use as is (No double interpolation)
                        self.get_variable(name).unwrap_or_default()
                    } else {
                        // Literal or Expression: Interpolate once
                        let text = self.get_token_value(&tokens[*i]);
                        self.interpolate_string(&text)
                    };
                    self.set_variable("Sbig_Response_Body".to_string(), val);
                    self.set_variable("Sbig_Response_File".to_string(), String::new());
                }
                TokenType::Point => {
                    *i += 1;
                    let val = self.get_token_value(&tokens[*i]);
                    self.current_status = val.parse::<u16>().unwrap_or(200);
                }
                TokenType::Note => {
                    *i += 1;
                    let key = self.get_token_value(&tokens[*i]);
                    *i += 1;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::As {
                        *i += 1;
                        let val = self.get_token_value(&tokens[*i]);
                        self.current_headers.insert(key, val);
                    }
                }
                TokenType::Identifier(ref s) if s == "file" => {
                    *i += 1;
                    let filename = self.get_token_value(&tokens[*i]);
                    self.set_variable("Sbig_Response_File".to_string(), filename);
                    self.set_variable("Sbig_Response_Body".to_string(), String::new());
                }
                _ => {}
            }
        }
    }
}
