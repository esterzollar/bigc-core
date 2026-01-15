use crate::bigdebug::BigDebug;
use crate::bignet::BigNet;
use crate::lexer::Lexer;
use crate::luck::BigLuck;
use crate::sound::SoundCommand;
use crate::tokens::{Token, TokenType};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::time::Instant;

mod actions;
mod architect;
pub mod biew;
mod bigweb;
mod bit;
mod bmath;
mod books;
mod control;
mod dbig;
mod dbr;
mod get;
mod helpers;
mod lists;
mod maps;
mod math_elements;

#[derive(Clone)]
pub struct Interpreter {
    pub variables: Arc<RwLock<HashMap<String, String>>>,
    pub functions: Arc<RwLock<HashMap<String, (Vec<String>, Vec<Token>)>>>,
    pub blueprints: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    pub views: Arc<RwLock<HashMap<String, Vec<Token>>>>,
    pub styles: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    pub assets: Arc<RwLock<HashMap<String, String>>>, 
    pub routes: Arc<RwLock<HashMap<String, String>>>, 
    pub event_queue: Arc<RwLock<VecDeque<String>>>,
    pub last_widget_clicked: Arc<RwLock<bool>>, 
    pub global_clicked: Arc<RwLock<bool>>,
    pub clicked_tags: Arc<RwLock<HashMap<String, bool>>>,
    pub last_clicked_tags: Arc<RwLock<HashMap<String, bool>>>,
    pub hovered_tags: Arc<RwLock<HashMap<String, bool>>>,
    pub pressed_tags: Arc<RwLock<HashMap<String, bool>>>,
    pub dragged_tags: Arc<RwLock<HashMap<String, (f32, f32)>>>, 

    pub last_hovered_tags: Arc<RwLock<HashMap<String, bool>>>,
    pub last_pressed_tags: Arc<RwLock<HashMap<String, bool>>>,
    pub last_dragged_tags: Arc<RwLock<HashMap<String, (f32, f32)>>>,
    pub last_drag_delta: Arc<RwLock<(f32, f32)>>,
    pub frame_drag_delta: Arc<RwLock<(f32, f32)>>,

    pub virtual_resolution: Option<(f32, f32)>,
    pub global_scale: f32,
    pub global_offset: (f32, f32),

    pub pending_doing: Arc<RwLock<Option<String>>>,
    pub mouse_pos: Arc<RwLock<(f32, f32)>>,
    pub keys_down: Arc<RwLock<HashMap<String, bool>>>,
    pub sound_tx: Option<Sender<SoundCommand>>,

    pub last_condition_met: bool,
    pub loop_stack: Vec<LoopInfo>,
    pub net: BigNet,
    pub luck: BigLuck,

    pub sql_enabled: bool,
    pub sbig_enabled: bool,
    pub pybig_enabled: bool,
    pub guy_enabled: bool,
    pub autolayering_enabled: bool,

    pub last_bug_found: bool,
    pub last_bug_type: String,

    pub return_triggered: bool,

    pub max_workers: usize,
    pub rate_limit: usize,
    pub call_depth: usize,
    pub local_scopes: Vec<HashMap<String, String>>, 
    pub current_status: u16,
    pub current_headers: HashMap<String, String>,
    pub log_file: Option<String>,
    pub ssl_config: Option<(String, String)>,

    pub start_time: Instant,
    pub last_delta_tick: Instant,
    pub current_dt: f32,                         // Added for stable animations
    pub call_stack: Vec<(String, usize, usize)>, // (BlockName, Line, Column)
    pub last_error_pos: Option<(usize, usize)>,
    pub last_line_pos: (usize, usize),
    pub full_source: String,

    // BIGPACK RUNTIME
    pub mounted_archive: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub start_index: usize,
    pub count: usize,
    pub limit: usize,
    pub is_valid: bool,
    // For-Each Fields
    pub is_foreach: bool,
    pub foreach_list: Vec<String>,
    pub foreach_keys: Vec<String>,   // For Maps
    pub foreach_var: String,         // Item or Key
    pub foreach_val_var: String,     // Value (for Maps)
    pub foreach_source_json: String, // Source for Map lookups
    pub foreach_idx: usize,
}


impl Interpreter {
    pub fn run_guy_direct(&mut self, filename: &str, tokens: Vec<Token>) {
        crate::guy_engine::run_direct(self, filename, tokens);
    }

    pub fn handle_start_guy(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        crate::guy_engine::handle_start(self, i, tokens);
    }

    pub fn consume_math(&mut self, i: &mut usize, tokens: &Vec<Token>) -> f32 {
        let mut math_tokens: Vec<Token> = Vec::new();
        let line = tokens[*i].line;
        while *i < tokens.len() && tokens[*i].line == line {
            let t = &tokens[*i];

            // Stop at UI keywords
            if let TokenType::Identifier(ref s) = t.token_type {
                if s == "size"
                    || s == "font"
                    || s == "radius"
                    || s == "fill"
                    || s == "color"
                    || s == "fontsize"
                    || s == "width"
                    || s == "tag"
                    || s == "layer"
                {
                    break;
                }
            }
            if matches!(
                t.token_type,
                TokenType::At
                    | TokenType::AtWord
                    | TokenType::Ampersand
                    | TokenType::Fill
                    | TokenType::Stroke
                    | TokenType::Spacing
                    | TokenType::Padding
                    | TokenType::Font
                    | TokenType::Rotate
                    | TokenType::Scale
                    | TokenType::Alpha
                    | TokenType::Tint
                    | TokenType::Layer
                    | TokenType::Tag
            ) {
                break;
            }

            // CRITICAL: Stop if we already have a value and the NEXT token is another value/variable
            // (meaning they are separate arguments like X and Y)
            if !math_tokens.is_empty() {
                let last_token = math_tokens.last().unwrap();
                let last_is_op = last_token.is_operator();
                let last_is_dollar = last_token.token_type == TokenType::Dollar;

                if !last_is_op && !last_is_dollar {
                    // Current is a value, last was a value. Stop!
                    if t.is_number()
                        || matches!(t.token_type, TokenType::Identifier(_) | TokenType::Dollar)
                    {
                        break;
                    }
                }
            }

            math_tokens.push(t.clone());
            *i += 1;
        }

        let resolved_expr = self.get_tokens_raw_value(&math_tokens);
        let mut lexer = crate::lexer::Lexer::new(&resolved_expr);
        let resolved_tokens = lexer.tokenize();
        

        // We don't need to step back here because the loop condition handled the stop
        self.evaluate_math_recursive(&resolved_tokens) as f32
    }

    pub fn new() -> Self {
        
        Interpreter {
            variables: Arc::new(RwLock::new(HashMap::new())),
            functions: Arc::new(RwLock::new(HashMap::new())),
            blueprints: Arc::new(RwLock::new(HashMap::new())),
            views: Arc::new(RwLock::new(HashMap::new())),
            styles: Arc::new(RwLock::new(HashMap::new())),
            assets: Arc::new(RwLock::new(HashMap::new())),
            routes: Arc::new(RwLock::new(HashMap::new())),
            event_queue: Arc::new(RwLock::new(VecDeque::new())),
            last_widget_clicked: Arc::new(RwLock::new(false)),
            global_clicked: Arc::new(RwLock::new(false)),
            clicked_tags: Arc::new(RwLock::new(HashMap::new())),
            last_clicked_tags: Arc::new(RwLock::new(HashMap::new())),
            hovered_tags: Arc::new(RwLock::new(HashMap::new())),
            pressed_tags: Arc::new(RwLock::new(HashMap::new())),
            dragged_tags: Arc::new(RwLock::new(HashMap::new())),
            last_hovered_tags: Arc::new(RwLock::new(HashMap::new())),
            last_pressed_tags: Arc::new(RwLock::new(HashMap::new())),
            last_dragged_tags: Arc::new(RwLock::new(HashMap::new())),
            last_drag_delta: Arc::new(RwLock::new((0.0, 0.0))),
            frame_drag_delta: Arc::new(RwLock::new((0.0, 0.0))),

            virtual_resolution: None,
            global_scale: 1.0,
            global_offset: (0.0, 0.0),

            pending_doing: Arc::new(RwLock::new(None)),
            mouse_pos: Arc::new(RwLock::new((0.0, 0.0))),
            keys_down: Arc::new(RwLock::new(HashMap::new())),
            sound_tx: None,

            last_condition_met: true,
            loop_stack: Vec::new(),
            net: BigNet::new(),
            luck: BigLuck::new(),
            sql_enabled: false,
            sbig_enabled: false,
            pybig_enabled: false,
            guy_enabled: false,
            autolayering_enabled: false,
            last_bug_found: false,
            last_bug_type: String::new(),
            return_triggered: false,
            max_workers: 1,
            rate_limit: 0,
            call_depth: 0,
            local_scopes: Vec::new(),
            current_status: 200,
            current_headers: HashMap::new(),
            log_file: None,
            ssl_config: None,
            start_time: Instant::now(),
            last_delta_tick: Instant::now(),
            current_dt: 0.016,
            call_stack: Vec::new(),
            last_error_pos: None,
            last_line_pos: (1, 1),
            full_source: String::new(),
            mounted_archive: None,
        }
    }

    pub fn resolve_file(&self, path: &str) -> Option<Vec<u8>> {
        // 1. Try Archive
        if let Some(archive) = &self.mounted_archive {
            if let Some(data) = crate::bigpack::BigPack::fetch(archive, path) {
                return Some(data);
            }
        }

        // 2. Try Disk
        if let Ok(data) = fs::read(path) {
            return Some(data);
        }

        None
    }

    pub fn resolve_file_as_string(&self, path: &str) -> Option<String> {
        if let Some(bytes) = self.resolve_file(path) {
            return String::from_utf8(bytes).ok();
        }
        None
    }

    pub fn heal_tokens(&self, tokens: Vec<Token>) -> Vec<Token> {
        let mut healed: Vec<Token> = Vec::new();
        let mut loop_starts: Vec<usize> = Vec::new(); // indices in 'healed'

        let mut i = 0;
        while i < tokens.len() {
            let t = &tokens[i];
            let col = t.column;
            let line = t.line;

            // Detect if we just finished an indented block
            if i > 0 && line > tokens[i - 1].line {
                // Check loops
                while let Some(&start_idx) = loop_starts.last() {
                    let start_col = healed[start_idx].column;
                    if col <= start_col
                        && t.token_type != TokenType::Keep
                        && t.token_type != TokenType::EOF
                    {
                        // Guess: Loop ended here. Insert virtual Keep + True condition.
                        let virtual_line = tokens[i - 1].line + 1;
                        healed.push(Token::new(TokenType::Keep, virtual_line, start_col));
                        healed.push(Token::new(
                            TokenType::Char('1'),
                            virtual_line,
                            start_col + 1,
                        ));
                        loop_starts.pop();
                    } else {
                        break;
                    }
                }
            }

            // Track starts
            if t.token_type == TokenType::Start
                && i + 1 < tokens.len() {
                    if tokens[i + 1].token_type == TokenType::Loop { loop_starts.push(healed.len()) }
                }

            // Track manual ends to avoid double-closing
            if t.token_type == TokenType::Keep {
                loop_starts.pop();
            }

            healed.push(t.clone());
            i += 1;
        }

        // Final check: Close any remaining blocks at the end of file
        let last_line = tokens.last().map(|t| t.line).unwrap_or(0);
        while loop_starts.pop().is_some() {
            healed.push(Token::new(TokenType::Keep, last_line, 1));
        }

        healed
    }

    pub fn run(&mut self, tokens: Vec<Token>) {
        self.call_depth += 1;
        if self.call_depth > 100 {
            self.report_error("Stack Overflow! (Recursion Limit: 100)", 0, 0);
            self.call_depth -= 1;
            return;
        }

        let pushed_scope = if self.call_depth > 1 {
            self.local_scopes.push(HashMap::new());
            true
        } else {
            false
        };

        let mut i = 0;
        while i < tokens.len() {
            if self.return_triggered {
                break;
            }

            // --- DEBUG TRACE ---
            BigDebug::trace(self, &tokens[i]);

            let token_type = &tokens[i].token_type;

            // Track position of this specific token
            self.last_line_pos = (tokens[i].line, tokens[i].column);

            // --- AUTOMATIC ERROR HALT ---
            if self.last_bug_found {
                let mut is_checking = false;
                if *token_type == TokenType::If
                    && i + 3 < tokens.len()
                        && tokens[i + 1].token_type == TokenType::Any
                            && tokens[i + 2].token_type == TokenType::Bug
                            && tokens[i + 3].token_type == TokenType::Found
                        {
                            is_checking = true;
                        }

                if !is_checking {
                    println!(
                        "DEBUG HALT: Current Token: {:?} at Line {}",
                        token_type, tokens[i].line
                    );
                    if i + 1 < tokens.len() {
                        println!("DEBUG HALT: Next Token: {:?}", tokens[i + 1].token_type);
                    }

                    let msg = self.last_bug_type.clone();
                    self.report_error(&msg, tokens[i].line, tokens[i].column);
                    self.last_bug_found = false;
                    break;
                }
            }

            match token_type {
                // RETURN (Functional)
                TokenType::Return => {
                    i += 1;
                    if i < tokens.len() {
                        let val_raw = self.get_complex_value(&mut i, &tokens);
                        let val = self.interpolate_string(&val_raw);
                        self.set_variable(String::from("ReturnValue"), val);
                    }
                    self.return_triggered = true;
                    return;
                }

                // ACTIONS (io.rs)
                TokenType::Print
                | TokenType::Wait
                | TokenType::Take
                | TokenType::Ask
                | TokenType::Reset
                | TokenType::Attach
                | TokenType::UserAgent
                | TokenType::Proxy
                | TokenType::Header
                | TokenType::Split
                | TokenType::Build
                | TokenType::Event
                | TokenType::Replace
                | TokenType::Pack
                | TokenType::Unpack
                | TokenType::Command => {
                    self.handle_action(&mut i, &tokens);
                }

                TokenType::Use => {
                    if i + 1 < tokens.len() {
                        match &tokens[i + 1].token_type {
                            TokenType::Sql => {
                                self.handle_use_sql(&mut i, &tokens);
                            }
                            TokenType::PyBig => {
                                self.pybig_enabled = true;
                                println!("BigC: pyBig System Enabled (External Python Required)");
                            }
                            TokenType::Sbig => {
                                self.handle_use_sbig(&mut i, &tokens);
                            }
                            TokenType::Guy => {
                                self.guy_enabled = true;
                                if self.sound_tx.is_none() {
                                    self.sound_tx = Some(crate::sound::start_sound_engine());
                                }
                                println!("BigC: BigGuy Graphic Engine Unlocked.");
                            }
                            TokenType::Sound => {
                                if self.sound_tx.is_none() {
                                    self.sound_tx = Some(crate::sound::start_sound_engine());
                                }
                                println!("BigC: Audio Engine Unlocked.");
                            }
                            TokenType::Autolayering => {
                                self.autolayering_enabled = true;
                                println!("BigGuy: Auto-Layering System Enabled (Ren'Py Style).");
                            }
                            TokenType::Lab => {
                                i += 1; // Consume Lab token
                                if i + 1 < tokens.len() {
                                    if let TokenType::String(filename) = &tokens[i + 1].token_type {
                                        if let Some(lab_content) =
                                            self.resolve_file_as_string(filename)
                                        {
                                            let mut lab_lexer = Lexer::new(&lab_content);
                                            let lab_tokens = lab_lexer.tokenize();
                                            self.run(lab_tokens);
                                        }
                                    }
                                }
                            }
                            TokenType::String(filename) => {
                                // Shorthand: use "file.big"
                                if let Some(lab_content) = self.resolve_file_as_string(filename) {
                                    let mut lab_lexer = Lexer::new(&lab_content);
                                    let lab_tokens = lab_lexer.tokenize();
                                    self.run(lab_tokens);
                                }
                            }
                            TokenType::Identifier(ref s) => {
                                if s.to_lowercase() == "web" {
                                    self.handle_use_sbig(&mut i, &tokens);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Books (books.rs)
                TokenType::Open
                | TokenType::Write
                | TokenType::Add
                | TokenType::Delete
                | TokenType::Copy
                | TokenType::Move
                | TokenType::Create => {
                    self.handle_books(&mut i, &tokens);
                }

                // DBB (dbig.rs)
                TokenType::Dbig => {
                    self.handle_dbig(&mut i, &tokens);
                }

                // Bit (Crypto)
                TokenType::Bit => {
                    self.handle_bit(&mut i, &tokens);
                }

                // CONTROL (control.rs)
                TokenType::Start => {
                    if i + 1 < tokens.len() && tokens[i + 1].token_type == TokenType::Server {
                        self.handle_start_server(&mut i, &tokens);
                    } else if i + 1 < tokens.len() && tokens[i + 1].token_type == TokenType::Guy {
                        self.handle_start_guy(&mut i, &tokens);
                    } else {
                        self.handle_control(&mut i, &tokens);
                    }
                }

                TokenType::Keep
                | TokenType::If
                | TokenType::Or
                | TokenType::Addrun
                | TokenType::Stop
                | TokenType::Sloop
                | TokenType::Loops => {
                    self.handle_control(&mut i, &tokens);
                }

                // Sbig (bigweb.rs)
                TokenType::On => {
                    self.handle_on(&mut i, &tokens);
                }
                TokenType::Reply => {
                    self.handle_reply(&mut i, &tokens);
                }

                // Maps (maps.rs)
                TokenType::Map => {
                    self.handle_map(&mut i, &tokens);
                }

                // Lists (lists.rs)
                TokenType::List => {
                    self.handle_list(&mut i, &tokens);
                }

                // Regex (bmath.rs)
                TokenType::Bmath => {
                    self.handle_bmath(&mut i, &tokens);
                }

                TokenType::Control => {
                    self.handle_server_config(&mut i, &tokens);
                }

                TokenType::Pin => {
                    i += 1;
                    if let TokenType::Identifier(b_name) = &tokens[i].token_type {
                        let b_name = b_name.clone();
                        i += 1;
                        if i < tokens.len() && tokens[i].token_type == TokenType::To {
                            i += 1;
                            let var_name = self.get_token_value(&tokens[i]);

                            // Access Shared Blueprints
                            let lookup_name = b_name.to_lowercase();
                            let defaults_opt = {
                                if let Ok(bps) = self.blueprints.read() {
                                    bps.get(&lookup_name).cloned()
                                } else {
                                    None
                                }
                            };

                            if let Some(defaults) = defaults_opt {
                                let mut map_json = String::from("{ ");
                                for (idx, (k, v)) in defaults.iter().enumerate() {
                                    if idx > 0 {
                                        map_json.push(',');
                                    }
                                    map_json.push_str(&format!(
                                        "\"{}\":\"{}\"",
                                        k,
                                        v.replace("\"", "\\\"")
                                    ));
                                }
                                map_json.push('}');
                                self.set_variable(var_name, map_json);
                            }
                        }
                    }
                }

                TokenType::Run => {
                    if i + 1 < tokens.len() {
                        match &tokens[i + 1].token_type {
                            TokenType::Sql => self.handle_run_sql(&mut i, &tokens),
                            TokenType::Background => {
                                i += 2; // Skip run background
                                if i < tokens.len() {
                                    let func_name = self.get_token_value(&tokens[i]);

                                    let func_data = {
                                        if let Ok(funcs) = self.functions.read() {
                                            funcs.get(&func_name).cloned()
                                        } else {
                                            None
                                        }
                                    };

                                    if let Some((_, func_tokens)) = func_data {
                                        let mut clone = self.clone();
                                        std::thread::spawn(move || {
                                            clone.run(func_tokens);
                                        });
                                    }
                                }
                            }
                            _ => {
                                let name = self.get_token_raw_name(&tokens[i + 1]);
                                if !name.is_empty() {
                                    i += 2; // Skip Run and Name

                                    // 1. Check for Function Call with Arguments: run Name(arg1, arg2)
                                    if i < tokens.len() && tokens[i].token_type == TokenType::LParen
                                    {
                                        i += 1;
                                        let mut args = Vec::new();
                                        while i < tokens.len()
                                            && tokens[i].token_type != TokenType::RParen
                                        {
                                            let val = self.get_complex_value(&mut i, &tokens);
                                            args.push(self.interpolate_string(&val));
                                            if i < tokens.len()
                                                && tokens[i].token_type == TokenType::Char(',')
                                            {
                                                i += 1;
                                            }
                                        }
                                        if i < tokens.len()
                                            && tokens[i].token_type == TokenType::RParen
                                        {
                                            i += 1;
                                        }

                                        let func_data = {
                                            if let Ok(funcs) = self.functions.read() {
                                                funcs.get(&name).cloned()
                                            } else {
                                                None
                                            }
                                        };

                                        if let Some((params, func_tokens)) = func_data {
                                            self.call_depth += 1;
                                            self.local_scopes.push(HashMap::new());
                                            for (idx, p_name) in params.iter().enumerate() {
                                                if idx < args.len() {
                                                    if let Some(scope) =
                                                        self.local_scopes.last_mut()
                                                    {
                                                        scope.insert(
                                                            p_name.clone(),
                                                            args[idx].clone(),
                                                        );
                                                    }
                                                }
                                            }
                                            let (l, c) = if i < tokens.len() {
                                                (tokens[i].line, tokens[i].column)
                                            } else {
                                                (0, 0)
                                            };
                                            self.call_stack.push((name.clone(), l, c));
                                            self.run(func_tokens);
                                            self.call_stack.pop();
                                            self.return_triggered = false;
                                            self.local_scopes.pop();
                                            self.call_depth -= 1;
                                            let ret_val = self
                                                .variables
                                                .read()
                                                .unwrap()
                                                .get("ReturnValue")
                                                .cloned()
                                                .unwrap_or(String::from("nothing"));
                                            i = i.saturating_sub(1); // Align for set as
                                            self.handle_set_as_multiple(
                                                &mut i,
                                                &tokens,
                                                vec![ret_val],
                                            );
                                        }
                                    } else {
                                        // 2. Call as No-Parameter Function: run Name
                                        let func_data = {
                                            if let Ok(funcs) = self.functions.read() {
                                                funcs.get(&name).cloned()
                                            } else {
                                                None
                                            }
                                        };

                                        if let Some((_, func_tokens)) = func_data {
                                            self.call_depth += 1;
                                            self.local_scopes.push(HashMap::new());
                                            let (l, c) = if i < tokens.len() {
                                                (tokens[i].line, tokens[i].column)
                                            } else {
                                                (0, 0)
                                            };
                                            self.call_stack.push((name.clone(), l, c));
                                            self.run(func_tokens);
                                            self.call_stack.pop();
                                            self.return_triggered = false;
                                            self.local_scopes.pop();
                                            self.call_depth -= 1;
                                            let ret_val = self
                                                .variables
                                                .read()
                                                .unwrap()
                                                .get("ReturnValue")
                                                .cloned()
                                                .unwrap_or(String::from("nothing"));
                                            i = i.saturating_sub(1); // Align for set as
                                            self.handle_set_as_multiple(
                                                &mut i,
                                                &tokens,
                                                vec![ret_val],
                                            );
                                        }
                                    }
                                }
                                i -= 1; // Main loop will increment
                            }
                            _ => {}
                        }
                    }
                }

                // GET (get.rs)
                TokenType::Get | TokenType::Look | TokenType::Check => {
                    if *token_type == TokenType::Get
                        && i + 1 < tokens.len()
                        && tokens[i + 1].token_type == TokenType::Map
                    {
                        i += 1;
                        self.handle_set_as_multiple(&mut i, &tokens, vec![String::from("{}")]);
                    } else if *token_type == TokenType::Check {
                        self.handle_architect(&mut i, &tokens);
                    } else if i + 1 < tokens.len() && tokens[i + 1].token_type == TokenType::Sql {
                        self.handle_get_sql(&mut i, &tokens);
                    } else {
                        self.handle_get(&mut i, &tokens);
                    }
                }

                // BLUEPRINT or STYLE BLOCK: [Blueprint: Name] or [Style: Name]
                TokenType::LBracket => {
                    if i + 2 < tokens.len()
                        && (tokens[i + 1].token_type == TokenType::Blueprint
                            || tokens[i + 1].token_type == TokenType::Style)
                        && tokens[i + 2].token_type == TokenType::Colon
                    {
                        let is_style = tokens[i + 1].token_type == TokenType::Style;
                        i += 3;

                        // Capture the full name (including dots) until ]
                        let mut b_name = String::new();
                        while i < tokens.len() && tokens[i].token_type != TokenType::RBracket {
                            b_name.push_str(&self.get_token_value(&tokens[i]));
                            i += 1;
                        }

                        if i < tokens.len() && tokens[i].token_type == TokenType::RBracket {
                            i += 1;
                            let mut defaults = HashMap::new();
                            while i < tokens.len() {
                                // Look for + prop : val
                                if tokens[i].token_type == TokenType::Plus {
                                    i += 1;
                                    let prop_name = match &tokens[i].token_type {
                                        TokenType::Fill => Some(String::from("fill")),
                                        TokenType::Font => Some(String::from("font")),
                                        TokenType::Spacing => Some(String::from("spacing")),
                                        TokenType::Padding => Some(String::from("padding")),
                                        TokenType::Stroke => Some(String::from("stroke")),
                                        TokenType::Layer => Some(String::from("layer")),
                                        TokenType::Alpha => Some(String::from("alpha")),
                                        TokenType::Identifier(s) if s == "radius" => {
                                            Some(String::from("radius"))
                                        }
                                        TokenType::Identifier(s) if s == "border" => {
                                            Some(String::from("stroke_width"))
                                        }
                                        TokenType::Identifier(s) => Some(s.clone()),
                                        _ => None,
                                    };

                                    if let Some(prop) = prop_name {
                                        i += 1;
                                        if i < tokens.len()
                                            && tokens[i].token_type == TokenType::Colon
                                        {
                                            i += 1;
                                            let val = self.get_token_value(&tokens[i]);
                                            defaults.insert(prop.to_lowercase(), val);
                                        }
                                    }
                                } else if tokens[i].token_type == TokenType::EOF
                                    || tokens[i].token_type == TokenType::LBracket
                                    || tokens[i].token_type == TokenType::Pin
                                    || tokens[i].line > tokens[i - 1].line + 1
                                {
                                    // Stop at next block, or blank line (line jump > 1)
                                    i -= 1;
                                    break;
                                }
                                i += 1;
                            }

                            if is_style {
                                if let Ok(mut styles) = self.styles.write() {
                                    styles.insert(b_name.to_lowercase(), defaults);
                                }
                            } else if let Ok(mut bps) = self.blueprints.write() {
                                bps.insert(b_name.to_lowercase(), defaults);
                            }
                        }
                    }
                }

                TokenType::Global | TokenType::Update | TokenType::Set => {
                    let is_global = *token_type == TokenType::Global;
                    let is_set = *token_type == TokenType::Set;
                    i += 1;
                    if i < tokens.len() {
                        if is_set && tokens[i].token_type == TokenType::Volume {
                            i += 1;
                            let vol_str = self.get_complex_value(&mut i, &tokens);
                            let vol = vol_str.parse::<f32>().unwrap_or(1.0);
                            if let Some(tx) = &self.sound_tx {
                                let _ = tx.send(crate::sound::SoundCommand::SetVolume(vol));
                            }
                            i -= 1; // Main loop increment
                        } else if let TokenType::Identifier(name) = &tokens[i].token_type {
                            let var_name = name.clone();
                            if i + 1 < tokens.len() && tokens[i + 1].token_type == TokenType::Assign
                            {
                                // Delegate to Unified Handler
                                self.handle_assignment(&mut i, &tokens, var_name.clone());

                                // FORCE GLOBAL SYNC: Always move to shared map for 'global' keyword
                                if is_global {
                                    if let Some(val) = self.get_variable(&var_name) {
                                        if let Ok(mut vars) = self.variables.write() {
                                            vars.insert(var_name, val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                TokenType::PythonCode(code) => {
                    if self.pybig_enabled {
                        use std::io::Write;
                        use std::process::Command;

                        let temp_file = "pybig_temp.py";
                        let mut file = std::fs::File::create(temp_file)
                            .expect("Failed to create temp python file");
                        file.write_all(code.as_bytes())
                            .expect("Failed to write to temp python file");

                        let python_cmd = if cfg!(target_os = "windows") {
                            "python"
                        } else {
                            "python3"
                        };
                        let output =
                            Command::new(python_cmd)
                                .arg(temp_file)
                                .output()
                                .unwrap_or_else(|_| panic!("Failed to execute {}. Is it installed?",
                                    python_cmd));

                        if !output.stdout.is_empty() {
                            println!("{}", String::from_utf8_lossy(&output.stdout));
                        }
                        if !output.stderr.is_empty() {
                            eprintln!("PyBig Error: {}", String::from_utf8_lossy(&output.stderr));
                        }

                        let _ = std::fs::remove_file(temp_file);
                    } else {
                        println!("Big Error: pyBig is locked! Use 'use pyBig' first.");
                    }
                }

                // VARIABLE ASSIGNMENT or UNKNOWN KEYWORD
                _other => {
                    let var_name = self.get_token_raw_name(&tokens[i]);
                    // Check for Assignment: Name = Val
                    if i + 1 < tokens.len() && tokens[i + 1].token_type == TokenType::Assign {
                        self.handle_assignment(&mut i, &tokens, var_name);
                    } else if i + 1 < tokens.len() && tokens[i + 1].token_type == TokenType::Dot {
                        // Check for Dot Notation: Object.Prop = Val
                        let obj_name = var_name;
                        i += 2; // skip name and dot
                        if i < tokens.len() {
                            if let TokenType::Identifier(prop_name) = &tokens[i].token_type {
                                let p_name = prop_name.clone();
                                if i + 1 < tokens.len()
                                    && tokens[i + 1].token_type == TokenType::Assign
                                {
                                    i += 1; // move to assign
                                    self.handle_dot_assignment(&mut i, &tokens, obj_name, p_name);
                                }
                            }
                        }
                    }
                }
            }
            i += 1;
        }

        if pushed_scope {
            self.local_scopes.pop();
        }

        // Final check for errors before returning to caller
        if self.last_bug_found {
            let (l, c) = self.last_line_pos;
            self.report_error(&self.last_bug_type.clone(), l, c);
            self.last_bug_found = false;
        }

        self.call_depth -= 1;
    }

    pub fn get_variable(&self, name: &str) -> Option<String> {
        // SPECIAL: System Time Variables
        if name == "BigTick" || name == "bigtick" {
            return Some(self.start_time.elapsed().as_millis().to_string());
        }
        if name == "BigDelta" || name == "bigdelta" {
            return Some(self.current_dt.to_string());
        }
        if name == "MouseX" || name == "mousex" {
            return Some(self.mouse_pos.read().unwrap().0.round().to_string());
        }
        if name == "MouseY" || name == "mousey" {
            return Some(self.mouse_pos.read().unwrap().1.round().to_string());
        }
        if name == "DragX" || name == "dragx" {
            return Some(self.last_drag_delta.read().unwrap().0.to_string());
        }
        if name == "DragY" || name == "dragy" {
            return Some(self.last_drag_delta.read().unwrap().1.to_string());
        }

        // DOT NOTATION: Object.Property
        if name.contains('.') {
            let parts: Vec<&str> = name.split('.').collect();
            if parts.len() == 2 {
                let obj_name = parts[0];
                let prop_name = parts[1];

                if let Some(obj_json) = self.get_variable(obj_name) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&obj_json) {
                        if let Some(val) = parsed.get(prop_name) {
                            return Some(match val {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                _ => val.to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Check Local Scopes (Stack)
        for scope in self.local_scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
            if let Some(val) = scope.get(&name.to_lowercase()) {
                return Some(val.clone());
            }
        }

        // Check Global (Shared)
        if let Ok(vars) = self.variables.read() {
            if let Some(val) = vars.get(name) {
                return Some(val.clone());
            }
            // Fallback: Try lowercase (Handling Keyword case folding like 'File' -> 'file')
            if let Some(val) = vars.get(&name.to_lowercase()) {
                return Some(val.clone());
            }
        }

        None
    }

    pub fn set_variable(&mut self, name: String, value: String) {
        let name = if name.to_lowercase() == "returnvalue" {
            String::from("ReturnValue")
        } else {
            name
        };

        let is_debug = self.get_variable("BigDebug").unwrap_or_default() == "true";
        let old_val = self.get_variable(&name);

        // Special System Variables or Global Suffixes (Always Global/Shared)
        if name == "ReturnValue"
            || name == "Sbig_Response_Body"
            || name == "Sbig_Response_File"
            || name == "RequestBody"
            || name == "RequestPath"
            || name == "RequestMethod"
            || name == "RequestExtra"
            || name.ends_with("Raw")
            || name.ends_with("Content")
            || name.ends_with("Layout")
            || name.ends_with("Html")
            || name.ends_with("Biew")
            || name == "BugType"
        {
            if let Ok(mut vars) = self.variables.write() {
                BigDebug::log_var_change(&name, old_val.as_ref(), &value, is_debug);
                vars.insert(name, value);
            }
            return;
        }

        // REVOLUTION RULE: If inside a Doing, EVERYTHING is Local (Shadowing)
        if let Some(scope) = self.local_scopes.last_mut() {
            BigDebug::log_var_change(&name, old_val.as_ref(), &value, is_debug);
            scope.insert(name, value);
        } else {
            // Main script: Global (Shared)
            if let Ok(mut vars) = self.variables.write() {
                BigDebug::log_var_change(&name, old_val.as_ref(), &value, is_debug);
                vars.insert(name, value);
            }
        }
    }

    pub fn report_error(&mut self, message: &str, line: usize, col: usize) {
        println!("\n+--- BigC ERROR ---+");
        println!("| Message:  {}", message);
        println!("| Location: Line {}, Column {}", line, col);

        let lines: Vec<&str> = self.full_source.lines().collect();

        // Show the actual line where the error triggered
        if line > 0 && line <= lines.len() {
            println!("| Trigger:  {}", lines[line - 1].trim());
        }

        if !self.call_stack.is_empty() {
            println!("| Call Stack (Recent calls first):");
            for (name, l, c) in self.call_stack.iter().rev() {
                println!("|   > inside doing {} (Line {}, Col {})", name, l, c);
                if *l > 0 && *l <= lines.len() {
                    println!("|     Source: {}", lines[*l - 1].trim());
                }
            }
        }
        println!("+------------------+");
        self.last_error_pos = Some((line, col));
    }

    pub fn handle_dot_assignment(
        &mut self,
        i: &mut usize,
        tokens: &Vec<Token>,
        obj_name: String,
        prop_name: String,
    ) {
        *i += 1; // Move to value
        if *i >= tokens.len() {
            return;
        }

        let value_raw = self.get_token_value(&tokens[*i]);
        let value = self.interpolate_string(&value_raw);

        if let Some(obj_json) = self.get_variable(&obj_name) {
            if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(&obj_json) {
                if let Some(obj) = parsed.as_object_mut() {
                    obj.insert(prop_name, serde_json::Value::String(value));
                    self.set_variable(obj_name, parsed.to_string());
                }
            }
        }
    }
}
