use crate::lexer::Lexer;
use crate::tokens::{Token, TokenType};

pub struct Biew;

#[derive(Debug, Clone, PartialEq)]
enum ParserMode {
    Html,
    StyleContainer,
    CssRule,
    Raw,
}

struct ParserState {
    mode: ParserMode,
    html_stack: Vec<String>,
    css_depth: usize,
    output: String,
    style_content: String,
}

impl Biew {
    pub fn transpile_biew(content: &str) -> String {
        let mut state = ParserState {
            mode: ParserMode::Html,
            html_stack: Vec::new(),
            css_depth: 0,
            output: String::new(),
            style_content: String::new(),
        };

        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize();

        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let mut last_line = 0;
        for t in tokens {
            if t.token_type == TokenType::EOF {
                break;
            }
            if last_line != 0 && t.line != last_line {
                lines.push(current_line);
                current_line = Vec::new();
            }
            last_line = t.line;
            current_line.push(t);
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        let raw_lines: Vec<&str> = content.lines().collect();

        for line_tokens in lines {
            if line_tokens.is_empty() {
                continue;
            }
            let line_idx = line_tokens[0].line - 1;
            let raw_text = if line_idx < raw_lines.len() {
                raw_lines[line_idx]
            } else {
                ""
            };
            Self::process_line(&line_tokens, raw_text, &mut state);
        }

        while let Some(tag) = state.html_stack.pop() {
            state.output.push_str(&format!(
                "</{}>
",
                tag
            ));
        }

        let mut html = String::from(
            "<!DOCTYPE html>
<html>
<head>
<meta charset='UTF-8'>
",
        );
        if !state.style_content.is_empty() {
            html.push_str(
                "<style>
",
            );
            html.push_str(&state.style_content);
            html.push_str(
                "</style>
",
            );
        }
        html.push_str(
            "</head>
<body>
",
        );
        html.push_str(&state.output);
        html.push_str(
            "
</body>
</html>",
        );
        html
    }

    pub fn transpile_bss(content: &str) -> String {
        let mut state = ParserState {
            mode: ParserMode::StyleContainer,
            html_stack: Vec::new(),
            css_depth: 0,
            output: String::new(),
            style_content: String::new(),
        };

        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize();

        let mut lines = Vec::new();
        let mut current_line = Vec::new();
        let mut last_line = 0;
        for t in tokens {
            if t.token_type == TokenType::EOF {
                break;
            }
            if last_line != 0 && t.line != last_line {
                lines.push(current_line);
                current_line = Vec::new();
            }
            last_line = t.line;
            current_line.push(t);
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        let raw_lines: Vec<&str> = content.lines().collect();

        for line_tokens in lines {
            if line_tokens.is_empty() {
                continue;
            }
            let line_idx = line_tokens[0].line - 1;
            let raw_text = if line_idx < raw_lines.len() {
                raw_lines[line_idx]
            } else {
                ""
            };
            Self::process_line(&line_tokens, raw_text, &mut state);
        }

        state.style_content
    }

    fn process_line(tokens: &[Token], raw_line: &str, state: &mut ParserState) {
        let trimmed = raw_line.trim().to_lowercase();

        if state.mode == ParserMode::Raw {
            if trimmed == "end raw" {
                state.mode = ParserMode::Html;
            } else {
                state.output.push_str(&format!(
                    "{}
",
                    raw_line
                ));
            }
            return;
        }

        if trimmed == "raw" {
            state.mode = ParserMode::Raw;
            return;
        }
        if trimmed == "style" && state.mode == ParserMode::Html {
            state.mode = ParserMode::StyleContainer;
            return;
        }
        if trimmed == "end style" {
            state.mode = ParserMode::Html;
            state.css_depth = 0;
            return;
        }

        if tokens.is_empty() {
            return;
        }
        let first_text = Self::get_raw_text(&tokens[0]).to_lowercase();

        if first_text == "raw" && tokens.len() > 1 {
            let mut content = String::new();
            for (idx, t) in tokens[1..].iter().enumerate() {
                content.push_str(&Self::get_val(t));
                if idx + 1 < tokens[1..].len() {
                    let next_t = &tokens[idx + 2];
                    if Self::is_alphanumeric_token(t) && Self::is_alphanumeric_token(next_t) {
                        content.push(' ');
                    }
                }
            }
            let final_content = content.trim().to_string();
            if state.mode == ParserMode::Html {
                state.output.push_str(&format!(
                    "{}
",
                    final_content
                ));
            } else {
                state.style_content.push_str(&format!(
                    "{}
",
                    final_content
                ));
            }
            return;
        }

        match state.mode {
            ParserMode::Html => {
                if first_text == "end" {
                    if let Some(tag) = state.html_stack.pop() {
                        state.output.push_str(&format!(
                            "</{}>
",
                            tag
                        ));
                    }
                } else if tokens[0].token_type == TokenType::Less {
                    state.output.push_str(&format!(
                        "{}
",
                        raw_line
                    ));
                } else {
                    let (tag, attrs, content) = Self::parse_biew_tokens(tokens);
                    if tag.is_empty() {
                        return;
                    }
                    if Self::is_void(&tag) {
                        state.output.push_str(&format!(
                            "<{} {} />
",
                            tag, attrs
                        ));
                    } else if !content.is_empty() {
                        state.output.push_str(&format!(
                            "<{} {}>{}</{}>
",
                            tag, attrs, content, tag
                        ));
                    } else {
                        state.html_stack.push(tag.clone());
                        state.output.push_str(&format!(
                            "<{} {}>
",
                            tag, attrs
                        ));
                    }
                }
            }
            ParserMode::StyleContainer | ParserMode::CssRule => {
                if first_text == "style" && tokens.len() > 1 {
                    state.mode = ParserMode::CssRule;
                    state.css_depth += 1;
                    let mut selector = String::new();
                    for t in &tokens[1..] {
                        selector.push_str(&Self::get_val(t));
                        if let TokenType::Identifier(_) = t.token_type {
                            selector.push(' ');
                        }
                    }
                    let selector = selector.trim().to_string();
                    let final_sel = if selector.starts_with('@') {
                        format!("#{}", &selector[1..])
                    } else if selector.starts_with(':')
                        || selector.starts_with('.')
                        || selector.starts_with('#')
                        || selector.starts_with('[')
                    {
                        selector.clone()
                    } else {
                        let first_part = selector.split_whitespace().next().unwrap_or("");
                        if [
                            "body", "html", "div", "h1", "h2", "h3", "p", "a", "button", "img",
                            "li", "ul", "table", "tr", "td", "th", "span", "header", "footer",
                            "section", "main", "nav", "aside", "pre", "code",
                        ]
                        .contains(&first_part)
                        {
                            selector.clone()
                        } else {
                            format!(".{}", selector)
                        }
                    };
                    state.style_content.push_str(&format!(
                        "{} {{
",
                        final_sel
                    ));
                } else if (first_text == "on" || tokens[0].token_type == TokenType::On)
                    && tokens.len() > 1
                {
                    state.css_depth += 1;
                    let event = Self::get_val(&tokens[1]);
                    state.style_content.push_str(&format!(
                        "&:{} {{
",
                        event
                    ));
                } else if first_text == "end" {
                    if state.css_depth > 0 {
                        state.css_depth -= 1;
                        state.style_content.push_str(
                            "}
",
                        );
                        if state.css_depth == 0 {
                            state.mode = ParserMode::StyleContainer;
                        }
                    }
                } else {
                    state
                        .style_content
                        .push_str(&Self::transpile_bss_line(tokens));
                }
            }
            _ => {}
        }
    }

    fn transpile_bss_line(tokens: &[Token]) -> String {
        if tokens.is_empty() {
            return String::new();
        }

        let first_text = Self::get_raw_text(&tokens[0]).to_lowercase();
        if first_text == "end" || first_text == "on" || first_text == "style" {
            return String::new();
        }

        let mut prop = String::new();
        let mut i = 0;
        while i < tokens.len() {
            let t = &tokens[i];
            match &t.token_type {
                TokenType::Identifier(s) => {
                    prop.push_str(s);
                    if i + 1 < tokens.len() && tokens[i + 1].token_type != TokenType::Minus {
                        i += 1;
                        break;
                    }
                }
                TokenType::Minus => prop.push('-'),
                TokenType::Char(c) => prop.push(*c),
                _ => break,
            }
            i += 1;
        }
        if prop.is_empty() {
            return String::new();
        }
        let prop_lower = prop.to_lowercase();
        if prop_lower == "center" {
            return "  display: flex; flex-direction: column; justify-content: center; align-items: center;
".to_string();
        }

        let mut val = String::new();
        let val_tokens = &tokens[i..];
        for (idx, t) in val_tokens.iter().enumerate() {
            val.push_str(&Self::get_val(t));
            if idx + 1 < val_tokens.len() {
                let next_t = &val_tokens[idx + 1];
                if Self::is_alphanumeric_token(t) && Self::is_alphanumeric_token(next_t) {
                    val.push(' ');
                }
            }
        }
        if val.is_empty() {
            return String::new();
        }

        let needs_px = match prop_lower.as_str() {
            "width" | "height" | "padding" | "margin" | "font-size" | "border-radius" | "top"
            | "left" => true,
            _ => false,
        };
        let final_val = if needs_px && val.chars().all(|c| c.is_ascii_digit() || c == ' ') {
            val.split_whitespace()
                .map(|v| {
                    if v.chars().all(|c| c.is_ascii_digit()) {
                        format!("{}px", v)
                    } else {
                        v.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            val
        };
        format!(
            "  {}: {};
",
            prop, final_val
        )
    }

    fn is_alphanumeric_token(t: &Token) -> bool {
        match &t.token_type {
            TokenType::Identifier(_)
            | TokenType::Number(_)
            | TokenType::String(_)
            | TokenType::Dollar => true,
            _ => false,
        }
    }

    fn parse_biew_tokens(tokens: &[Token]) -> (String, String, String) {
        if tokens.is_empty() {
            return (String::new(), String::new(), String::new());
        }
        let first_text = Self::get_raw_text(&tokens[0]).to_lowercase();
        let tag = match first_text.as_str() {
            "box" => "div".to_string(),
            "header" => "h1".to_string(),
            "subheader" => "h2".to_string(),
            "title" => "h3".to_string(),
            "text" => "p".to_string(),
            "button" => "button".to_string(),
            "link" => "a".to_string(),
            "image" => "img".to_string(),
            "item" => "li".to_string(),
            _ => first_text,
        };
        let mut id = String::new();
        let mut class = String::new();
        let mut attrs = Vec::new();
        let mut content = String::new();
        let mut i = 1;
        while i < tokens.len() {
            match &tokens[i].token_type {
                TokenType::At => {
                    i += 1;
                    if i < tokens.len() {
                        id = Self::get_val(&tokens[i]);
                    }
                }
                TokenType::Identifier(ref s) if s == "class" => {
                    i += 1;
                    if i < tokens.len() {
                        class = Self::get_val(&tokens[i]);
                    }
                }
                TokenType::Identifier(ref s) if s == "url" => {
                    i += 1;
                    if i < tokens.len() {
                        attrs.push(format!("href='{}'", Self::get_val(&tokens[i])));
                    }
                }
                TokenType::Identifier(ref s) if s == "with" => {
                    i += 1;
                    if i + 1 < tokens.len() {
                        attrs.push(format!(
                            "{}='{}'",
                            Self::get_val(&tokens[i]),
                            Self::get_val(&tokens[i + 1])
                        ));
                        i += 1;
                    }
                }
                TokenType::String(ref s) => {
                    if content.is_empty() {
                        content = s.clone();
                    }
                }
                _ => {}
            }
            i += 1;
        }
        let mut attr_str = String::new();
        if !id.is_empty() {
            attr_str.push_str(&format!("id='{}' ", id));
        }
        if !class.is_empty() {
            attr_str.push_str(&format!("class='{}' ", class));
        }
        for a in attrs {
            attr_str.push_str(&format!("{} ", a));
        }
        (tag, attr_str.trim().to_string(), content)
    }

    fn is_void(tag: &str) -> bool {
        match tag {
            "br" | "hr" | "img" | "input" | "meta" | "link" => true,
            _ => false,
        }
    }

    fn get_val(token: &Token) -> String {
        match &token.token_type {
            TokenType::String(s) | TokenType::Identifier(s) => s.clone(),
            TokenType::Number(n) => n.to_string(),
            TokenType::Char(c) => c.to_string(),
            TokenType::Minus => "-".to_string(),
            TokenType::Plus => "+".to_string(),
            TokenType::Star => "*".to_string(),
            TokenType::Slash => "/".to_string(),
            TokenType::At => "@".to_string(),
            TokenType::Less => "<".to_string(),
            TokenType::Greater => ">".to_string(),
            TokenType::Assign => "=".to_string(),
            TokenType::Ampersand => "&".to_string(),
            TokenType::LBracket => "[".to_string(),
            TokenType::RBracket => "]".to_string(),
            TokenType::LParen => "(".to_string(),
            TokenType::RParen => ")".to_string(),
            TokenType::LBrace => "{".to_string(),
            TokenType::RBrace => "}".to_string(),
            TokenType::Dollar => "$".to_string(),
            _ => format!("{:?}", token.token_type).to_lowercase(),
        }
    }

    fn get_raw_text(token: &Token) -> String {
        match &token.token_type {
            TokenType::Identifier(s) | TokenType::String(s) => s.clone(),
            TokenType::Minus => "-".to_string(),
            TokenType::Plus => "+".to_string(),
            TokenType::Star => "*".to_string(),
            TokenType::Slash => "/".to_string(),
            TokenType::Char(c) => c.to_string(),
            _ => format!("{:?}", token.token_type).to_lowercase(),
        }
    }
}
