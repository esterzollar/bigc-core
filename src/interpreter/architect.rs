use super::Interpreter;
use crate::tokens::{Token, TokenType};
use std::path::Path;

impl Interpreter {
    pub fn handle_architect(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax: check if "file.txt" here & set as {Result}
        *i += 1; // Skip "check"

        if *i < tokens.len() && tokens[*i].token_type == TokenType::If {
            *i += 1;
            if *i < tokens.len() {
                // Get the filename (handles variables or strings)
                let path_raw = self.get_token_value(&tokens[*i]);
                let path_str = self.interpolate_string(&path_raw);

                *i += 1;
                if *i < tokens.len() && tokens[*i].token_type == TokenType::Here {
                    let exists = Path::new(&path_str).exists();
                    let result = if exists { "true" } else { "false" };

                    // Use the standard connector to save the result
                    self.handle_set_as_multiple(i, tokens, vec![result.to_string()]);
                }
            }
        }
    }
}
