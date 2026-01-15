use super::Interpreter;
use crate::tokens::{Token, TokenType};
use rusqlite::Connection;

impl Interpreter {
    pub fn handle_use_sql(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1;
        if *i < tokens.len() && tokens[*i].token_type == TokenType::Sql {
            self.sql_enabled = true;
        }
    }

    pub fn handle_run_sql(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax: run sql "query" on "db.file"
        *i += 2; // Skip "run" and "sql"

        if !self.sql_enabled {
            println!("Big Error: SQL engine is locked! You must write 'use sql' at the start of your file.");
            return;
        }

        if *i < tokens.len() {
            let query_raw = self.get_token_value(&tokens[*i]);
            let query = self.interpolate_string(&query_raw);

            *i += 1;
            if *i < tokens.len() && tokens[*i].token_type == TokenType::On {
                *i += 1;
                let db_path_raw = self.get_token_value(&tokens[*i]);
                let db_path = self.interpolate_string(&db_path_raw);

                if let Ok(conn) = Connection::open(db_path) {
                    if let Err(e) = conn.execute(&query, []) {
                        println!("Big Error: SQL Run Failed - {}", e);
                    }
                } else {
                    println!("Big Error: Failed to open database.");
                }
            }
        }
    }

    pub fn handle_get_sql(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax: get sql "query" on "db.file" & set as list {Result}
        *i += 2; // Skip "get" and "sql"

        if !self.sql_enabled {
            println!("Big Error: SQL engine is locked! You must write 'use sql' at the start of your file.");
            return;
        }

        if *i < tokens.len() {
            let query_raw = self.get_token_value(&tokens[*i]);
            let query = self.interpolate_string(&query_raw);

            *i += 1;
            if *i < tokens.len() && tokens[*i].token_type == TokenType::On {
                *i += 1;
                let db_path_raw = self.get_token_value(&tokens[*i]);
                let db_path = self.interpolate_string(&db_path_raw);

                if let Ok(conn) = Connection::open(db_path) {
                    if let Ok(mut stmt) = conn.prepare(&query) {
                        let rows = stmt.query_map([], |row| {
                            let val: String = row.get(0).unwrap_or(String::from(""));
                            Ok(val)
                        });

                        let mut results = Vec::new();
                        if let Ok(iter) = rows {
                            for s in iter.flatten() {
                                results.push(s);
                            }
                        }
                        self.handle_set_as_multiple(i, tokens, results);
                    } else {
                        println!("Big Error: SQL Prepare Failed");
                    }
                }
            }
        }
    }
}
