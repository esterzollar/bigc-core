use crate::lexer::Lexer;
use crate::tokens::TokenType;
use serde_json::Value;
use std::fs;

pub struct BigHelp;

impl BigHelp {
    pub fn whatis(keyword: &str) {
        let lookup = keyword.to_lowercase();

        // 1. Try to load from external textbook.json
        let asset_path = std::path::Path::new("assets").join("textbook.json");
        let json_content = fs::read_to_string(&asset_path).unwrap_or_else(|_| String::from("{}"));
        let textbook: Value =
            serde_json::from_str(&json_content).unwrap_or(Value::Object(serde_json::Map::new()));

        if let Some(entry) = textbook.get(&lookup) {
            let title = entry
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");
            let description = entry
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("No description.");
            let usage = entry
                .get("example")
                .and_then(|v| v.as_str())
                .unwrap_or("No example provided.");

            println!("\n+--- Textbook: {} ---+", title);
            println!("| Description: {}", description);
            println!("| Example:     {}", usage);
            println!("+----------------------------+");
        } else {
            println!("\n+--- BigHelp: {} ---+", keyword);
            println!("| I don't have a textbook entry for that keyword yet.");
            println!("| Check the official Wiki in deploy_wiki/ for the full manual.");
            println!("+----------------------------+");
        }
    }

    pub fn show(filename: &str) {
        let content = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(_) => {
                println!("Big Error: Could not open '{}'", filename);
                return;
            }
        };

        let mut lexer = Lexer::new(&content);
        let tokens = lexer.tokenize();

        let mut counts = std::collections::HashMap::new();
        let mut doing_names = Vec::new();
        let mut view_names = Vec::new();

        let mut i = 0;
        while i < tokens.len() {
            let t = &tokens[i];
            let name = format!("{:?}", t.token_type)
                .split('(')
                .next()
                .unwrap()
                .to_string();
            *counts.entry(name).or_insert(0) += 1;

            if t.token_type == TokenType::Doing && i + 1 < tokens.len() {
                if let TokenType::Identifier(name) = &tokens[i + 1].token_type {
                    doing_names.push(name.clone());
                }
            }
            if t.token_type == TokenType::View && i + 1 < tokens.len() {
                if let TokenType::Identifier(name) = &tokens[i + 1].token_type {
                    view_names.push(name.clone());
                }
            }
            i += 1;
        }

        println!("\n--- Analysis for: {} ---", filename);
        println!("Lines:   {}", content.lines().count());
        println!(
            "Logic:   {} if/or blocks",
            counts.get("If").unwrap_or(&0) + counts.get("Or").unwrap_or(&0)
        );
        println!(
            "Doing:   {} defined blocks: {:?}",
            doing_names.len(),
            doing_names
        );
        println!(
            "Views:   {} defined screens: {:?}",
            view_names.len(),
            view_names
        );
        println!(
            "GUI:     {} draw commands",
            counts.get("Draw").unwrap_or(&0)
        );
        println!(
            "Math:    {} math operations",
            counts.get("Plus").unwrap_or(&0)
                + counts.get("Minus").unwrap_or(&0)
                + counts.get("Star").unwrap_or(&0)
        );
        println!("----------------------------------");
    }
}
