use crate::tokens::{Token, TokenType};

pub struct Lexer {
    pub(crate) input: Vec<char>,
    pub(crate) pos: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) {
        if self.pos < self.input.len() {
            if self.input[self.pos] == '\n' {
                self.line += 1;
                self.column = 0; // Will become 1 after increment
            }
            self.pos += 1;
            self.column += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.pos + 1 < self.input.len() {
            Some(self.input[self.pos + 1])
        } else {
            None
        }
    }

    pub fn advance_char_raw(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let c = self.input[self.pos];
            if c == '\n' {
                self.line += 1;
                self.column = 0;
            } else if c == '\t' {
                self.column += 3; // Keep the tab fix
            }
            self.pos += 1;
            self.column += 1;
            Some(c)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                }
                '#' => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                '!' => {
                    if let Some(next) = self.peek_next() {
                        if next == '=' {
                            tokens.push(Token::new(TokenType::NotEqual, self.line, self.column));
                            self.advance(); // consume !
                            self.advance(); // consume =
                        } else {
                            tokens.push(Token::new(TokenType::Char('!'), self.line, self.column));
                            self.advance();
                        }
                    } else {
                        tokens.push(Token::new(TokenType::Char('!'), self.line, self.column));
                        self.advance();
                    }
                }
                '!' => {
                    if let Some(next) = self.peek_next() {
                        if next == '=' {
                            tokens.push(Token::new(TokenType::NotEqual, self.line, self.column));
                            self.advance(); // consume !
                            self.advance(); // consume =
                        } else {
                            tokens.push(Token::new(TokenType::Char('!'), self.line, self.column));
                            self.advance();
                        }
                    } else {
                        tokens.push(Token::new(TokenType::Char('!'), self.line, self.column));
                        self.advance();
                    }
                }
                '=' => {
                    if let Some(next) = self.peek_next() {
                        if next == 'x' {
                            tokens.push(Token::new(TokenType::NotEqual, self.line, self.column));
                            self.advance(); // consume current =
                            self.advance(); // consume x
                        } else {
                            tokens.push(Token::new(TokenType::Assign, self.line, self.column));
                            self.advance();
                        }
                    } else {
                        tokens.push(Token::new(TokenType::Assign, self.line, self.column));
                        self.advance();
                    }
                }
                '&' => {
                    tokens.push(Token::new(TokenType::Ampersand, self.line, self.column));
                    self.advance();
                }
                '+' => {
                    tokens.push(Token::new(TokenType::Plus, self.line, self.column));
                    self.advance();
                }
                '-' => {
                    tokens.push(Token::new(TokenType::Minus, self.line, self.column));
                    self.advance();
                }
                '*' => {
                    tokens.push(Token::new(TokenType::Star, self.line, self.column));
                    self.advance();
                }
                '/' => {
                    tokens.push(Token::new(TokenType::Slash, self.line, self.column));
                    self.advance();
                }
                '^' => {
                    tokens.push(Token::new(TokenType::Caret, self.line, self.column));
                    self.advance();
                }
                '>' => {
                    if let Some(next) = self.peek_next() {
                        if next == '=' {
                            tokens.push(Token::new(
                                TokenType::GreaterEqual,
                                self.line,
                                self.column,
                            ));
                            self.advance(); // consume >
                            self.advance(); // consume =
                        } else {
                            tokens.push(Token::new(TokenType::Greater, self.line, self.column));
                            self.advance();
                        }
                    } else {
                        tokens.push(Token::new(TokenType::Greater, self.line, self.column));
                        self.advance();
                    }
                }
                '<' => {
                    if let Some(next) = self.peek_next() {
                        if next == '=' {
                            tokens.push(Token::new(TokenType::LessEqual, self.line, self.column));
                            self.advance(); // consume <
                            self.advance(); // consume =
                        } else {
                            tokens.push(Token::new(TokenType::Less, self.line, self.column));
                            self.advance();
                        }
                    } else {
                        tokens.push(Token::new(TokenType::Less, self.line, self.column));
                        self.advance();
                    }
                }
                '{' => {
                    tokens.push(Token::new(TokenType::LBrace, self.line, self.column));
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::new(TokenType::RBrace, self.line, self.column));
                    self.advance();
                }
                '(' => {
                    tokens.push(Token::new(TokenType::LParen, self.line, self.column));
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::new(TokenType::RParen, self.line, self.column));
                    self.advance();
                }
                '[' => {
                    tokens.push(Token::new(TokenType::LBracket, self.line, self.column));
                    self.advance();
                }
                ']' => {
                    tokens.push(Token::new(TokenType::RBracket, self.line, self.column));
                    self.advance();
                }
                '.' => {
                    tokens.push(Token::new(TokenType::Dot, self.line, self.column));
                    self.advance();
                }
                ':' => {
                    tokens.push(Token::new(TokenType::Colon, self.line, self.column));
                    self.advance();
                }
                '$' => {
                    tokens.push(Token::new(TokenType::Dollar, self.line, self.column));
                    self.advance();
                }
                '@' => {
                    tokens.push(Token::new(TokenType::At, self.line, self.column));
                    self.advance();
                }
                '"' => {
                    tokens.push(self.read_string());
                }
                c if c.is_ascii_digit() => {
                    tokens.push(self.read_number());
                }
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_identifier());
                }
                c => {
                    tokens.push(Token::new(TokenType::Char(c), self.line, self.column));
                    self.advance();
                }
            }
        }

        tokens.push(Token::new(TokenType::EOF, self.line, self.column));
        tokens
    }

    fn read_string(&mut self) -> Token {
        let start_col = self.column;
        self.advance(); // skip "
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                break;
            }

            if c == '\\' {
                // Corrected escape for backslash
                self.advance(); // consume backslash
                if let Some(next_c) = self.peek() {
                    match next_c {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'), // Corrected escape for backslash
                        '"' => s.push('"'),
                        _ => {
                            s.push('\\'); // Corrected escape for backslash
                            s.push(next_c);
                        }
                    }
                    self.advance();
                }
            } else {
                s.push(c);
                self.advance();
            }
        }
        self.advance(); // skip "
        Token::new(TokenType::String(s), self.line, start_col)
    }

    fn read_number(&mut self) -> Token {
        let start_col = self.column;
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let val = s.parse::<f64>().unwrap_or(0.0);

        // Check for 'x' suffix (SpeedVal)
        if let Some(c) = self.peek() {
            if c == 'x' || c == 'X' {
                self.advance(); // consume x
                return Token::new(TokenType::SpeedVal(val), self.line, start_col);
            }
        }

        Token::new(TokenType::Number(val), self.line, start_col)
    }

    fn read_identifier(&mut self) -> Token {
        let start_col = self.column;
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                s.push(c);
                self.advance();
            } else if c == '.' {
                // Peek ahead for s.loop or loop.s
                let mut peek_pos = self.pos + 1;
                let mut next_part = String::new();
                while peek_pos < self.input.len() && self.input[peek_pos].is_alphanumeric() {
                    next_part.push(self.input[peek_pos]);
                    peek_pos += 1;
                }

                let combined = format!("{}.{}", s, next_part);
                if combined == "s.loop" || combined == "loop.s" || combined == "k.loop" {
                    s = combined;
                    for _ in 0..next_part.len() + 1 {
                        self.advance();
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let token_type = match s.to_lowercase().as_str() {
            "init" => TokenType::Init,
            "step" => TokenType::Step,
            "towards" => TokenType::Towards,
            "speed" => TokenType::Speed,
            "rotate" => TokenType::Rotate,
            "scale" => TokenType::Scale,
            "alpha" => TokenType::Alpha,
            "tint" => TokenType::Tint,
            "layer" => TokenType::Layer,
            "clip" => TokenType::Clip,
            "tag" => TokenType::Tag,
            "print" => TokenType::Print,
            "update" => TokenType::Update,
            "get" => TokenType::Get,
            "set" => TokenType::Set,
            "global" => TokenType::Global,
            "as" => TokenType::As,
            "if" => TokenType::If,
            "or" => TokenType::Or,
            "take" => TokenType::Take,
            "wait" => TokenType::Wait,
            "type" => TokenType::Type,
            "export" => TokenType::Export,
            "use" => TokenType::Use,
            "lab" => TokenType::Lab,
            "attach" => TokenType::Attach,
            "stop" => TokenType::Stop,
            "warp" => TokenType::Warp,
            "run" => TokenType::Run,
            "start" => TokenType::Start,
            "build" => TokenType::Build,
            "pack" => TokenType::Pack,
            "unpack" => TokenType::Unpack,
            "command" => TokenType::Command,
            "bignet" => TokenType::Bignet,
            "netloop" => TokenType::Netloop,
            "look" => TokenType::Look,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "with" => TokenType::With,
            "proxy" => TokenType::Proxy,
            "user-agent" => TokenType::UserAgent,
            "header" => TokenType::Header,
            "post" => TokenType::Post,
            "split" => TokenType::Split,
            "by" => TokenType::By,
            "json" => TokenType::Json,
            "luck" => TokenType::Luck,
            "random" => TokenType::Random,
            "email" => TokenType::Email,
            "uuid" => TokenType::UUID,
            "loop" => TokenType::Loop,
            "s.loop" => TokenType::Sloop,
            "loop.s" => TokenType::Loops,
            "keep" => TokenType::Keep,
            "doing" => TokenType::Doing,
            "return" => TokenType::Return,
            "blueprint" => TokenType::Blueprint,
            "pin" => TokenType::Pin,
            "new" => TokenType::New,
            "addrun" => TokenType::Addrun,
            "andrun" => TokenType::Andrun,
            "end" => TokenType::End,
            "any" => TokenType::Any,
            "bug" => TokenType::Bug,
            "found" => TokenType::Found,
            "reset" => TokenType::Reset,
            "len" => TokenType::Len,
            "bigtick" => TokenType::BigTick,
            "bigdelta" => TokenType::BigDelta,
            "event" => TokenType::Event,
            "push" => TokenType::Push,
            "pop" => TokenType::Pop,
            "open" => TokenType::Open,
            "book" => TokenType::Book,
            "write" => TokenType::Write,
            "delete" => TokenType::Delete,
            "copy" => TokenType::Copy,
            "create" => TokenType::Create,
            "folder" => TokenType::Folder,
            "file" => TokenType::File,
            "to" => TokenType::To,
            "add" => TokenType::Add,
            "list" => TokenType::List,
            "from" => TokenType::From,
            "of" => TokenType::Of,
            "sort" => TokenType::Sort,
            "insert" => TokenType::Insert,
            "merge" => TokenType::Merge,
            "check" => TokenType::Check,
            "here" => TokenType::Here,
            "cut" => TokenType::Cut,
            "at" => TokenType::AtWord,
            "sql" => TokenType::Sql,
            "on" => TokenType::On,
            "dbig" => TokenType::Dbig,
            "remove" => TokenType::Remove,
            "keys" => TokenType::Keys,
            "value" => TokenType::Value,
            "guy" => TokenType::Guy,
            "view" => TokenType::View,
            "refresh" => TokenType::Refresh,
            "draw" => TokenType::Draw,
            "rectangle" => TokenType::Rectangle,
            "rounded" => TokenType::Rounded,
            "circle" => TokenType::Circle,
            "triangle" => TokenType::Triangle,
            "line" => TokenType::Line,
            "path" => TokenType::Path,
            "button" => TokenType::Button,
            "move" => TokenType::Move,
            "curve" => TokenType::Curve,
            "close" => TokenType::Close,
            "fill" => TokenType::Fill,
            "stroke" => TokenType::Stroke,
            "text" => TokenType::Text,
            "font" => TokenType::Font,
            "image" => TokenType::Image,
            "asset" => TokenType::Asset,
            "load" => TokenType::Load,
            "style" => TokenType::Style,
            "window" => TokenType::Window,
            "title" => TokenType::Title,
            "click" => TokenType::Click,
            "play" => TokenType::Play,
            "sound" => TokenType::Sound,
            "volume" => TokenType::Volume,
            "beep" => TokenType::Beep,
            "hover" => TokenType::Hover,
            "press" => TokenType::Press,
            "drag" => TokenType::Drag,
            "mouse" => TokenType::Mouse,
            "resize" => TokenType::Resize,
            "frame" => TokenType::Frame,
            "bind" => TokenType::Bind,
            "state" => TokenType::State,
            "padding" => TokenType::Padding,
            "spacing" => TokenType::Spacing,
            "row" => TokenType::Row,
            "col" => TokenType::Column,
            "column" => TokenType::Column,
            "scroll" => TokenType::Scroll,
            "area" => TokenType::Area,
            "btnclick" => TokenType::BtnClick,
            "keydown" => TokenType::KeyDown,
            "ask" => TokenType::Ask,
            "input" => TokenType::Input,
            "server" => TokenType::Server,
            "sbig" => TokenType::Sbig,
            "reply" => TokenType::Reply,
            "body" => TokenType::Body,
            "control" => TokenType::Control,
            "workers" => TokenType::Workers,
            "limit" => TokenType::Limit,
            "per" => TokenType::Per,
            "mins" => TokenType::Mins,
            "background" => TokenType::Background,
            "bit" => TokenType::Bit,
            "aes" => TokenType::Aes,
            "code" => TokenType::Code,
            "decode" => TokenType::Decode,
            "encrypt" => TokenType::Encrypt,
            "decrypt" => TokenType::Decrypt,
            "key" => TokenType::Key,
            "iv" => TokenType::Iv,
            "nebc" => TokenType::Nebc,
            "demon" => TokenType::Demon,
            "map" => TokenType::Map,
            "bmath" => TokenType::Bmath,
            "markdown" => TokenType::Markdown,
            "setting" => TokenType::Setting,
            "task" => TokenType::Task,
            "point" => TokenType::Point,
            "note" => TokenType::Note,
            "ssl" => TokenType::SSL,
            "record" => TokenType::Record,
            "pybig" => TokenType::PyBig,
            "solve" | "s" => TokenType::Solve,
            "remainder" => TokenType::Remainder,
            "sqrt" => TokenType::Sqrt,
            "sin" => TokenType::Sin,
            "cos" => TokenType::Cos,
            "tan" => TokenType::Tan,
            "abs" => TokenType::Abs,
            "floor" => TokenType::Floor,
            "ceil" => TokenType::Ceil,
            "round" => TokenType::Round,
            "clean" => TokenType::Clean,
            "bigcap" => TokenType::Bigcap,
            "lower" => TokenType::Lower,
            "replace" => TokenType::Replace,
            "with" => TokenType::With,
            "smaller" => TokenType::Smaller,
            "bigger" => TokenType::Bigger,
            "between" => TokenType::Between,
            "positive" => TokenType::Positive,
            "autolayering" => TokenType::Autolayering,
            "log" => TokenType::Log,
            "minimum" => TokenType::Minimum,
            "maximum" => TokenType::Maximum,
            "pi" => TokenType::PI,
            "euler" => TokenType::Euler,
            _ => TokenType::Identifier(s.clone()),
        };

        if s == "python3" {
            let saved_pos = self.pos;
            let saved_col = self.column;
            while let Some(c) = self.peek() {
                if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
                    self.advance();
                } else {
                    break;
                }
            }
            let mut next_word = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() {
                    next_word.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if next_word == "start" {
                return self.read_python_block();
            } else {
                self.pos = saved_pos;
                self.column = saved_col;
            }
        }
        Token::new(token_type, self.line, start_col)
    }

    fn read_python_block(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.column;
        let mut code = String::new();
        while let Some(c) = self.advance_char_raw() {
            code.push(c);
            if code.trim_end().ends_with("python3 end") {
                if let Some(idx) = code.rfind("python3") {
                    code.truncate(idx);
                }
                return Token::new(
                    TokenType::PythonCode(code.trim().to_string()),
                    start_line,
                    start_col,
                );
            }
        }
        Token::new(TokenType::PythonCode(code), start_line, start_col)
    }
}
