use super::Interpreter;
use crate::tokens::{Token, TokenType};
use regex::Regex;

impl Interpreter {
    pub fn handle_bmath(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        // Syntax: bmath "Text" @"Pattern" & set as {Result}
        *i += 1; // Skip "bmath"

        if *i < tokens.len() {
            let text = self.get_token_value(&tokens[*i]);
            *i += 1;

            if *i < tokens.len() && tokens[*i].token_type == TokenType::At {
                if !self.validate_at_strictness(*i, tokens) {
                    return;
                }
                *i += 1; // Skip @
                if *i < tokens.len() {
                    let pattern = self.get_token_value(&tokens[*i]);

                    let is_match = match Regex::new(&pattern) {
                        Ok(re) => re.is_match(&text),
                        Err(_) => false, // or print error?
                    };

                    let result = if is_match { "true" } else { "false" };
                    self.handle_set_as_multiple(i, tokens, vec![result.to_string()]);
                }
            }
        }
    }
}
