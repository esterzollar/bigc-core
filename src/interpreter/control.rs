use super::{Interpreter, LoopInfo};
use crate::tokens::{Token, TokenType};
use std::collections::HashMap;

impl Interpreter {
    pub fn handle_control(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        let token_type = tokens[*i].token_type.clone();
        let current_indent = tokens[*i].column;

        match token_type {
            TokenType::Step => self.handle_step(i, tokens),
            TokenType::If => {
                *i += 1;
                let mut condition_tokens = Vec::new();
                while *i < tokens.len()
                    && tokens[*i].token_type != TokenType::Ampersand
                    && tokens[*i].line == tokens[*i - 1].line
                {
                    condition_tokens.push(tokens[*i].clone());
                    *i += 1;
                }

                let is_bug_check = condition_tokens.len() == 3
                    && condition_tokens[0].token_type == TokenType::Any
                    && condition_tokens[1].token_type == TokenType::Bug
                    && condition_tokens[2].token_type == TokenType::Found;

                let res = self.evaluate_condition(&condition_tokens);
                if is_bug_check {
                    self.last_bug_found = false;
                }
                self.last_condition_met = res;

                if res {
                    // Logic is true: move to next token (on same line or next indented line)
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::Ampersand {
                        *i += 1;
                    }
                    *i -= 1; // Main loop will increment
                } else {
                    // Logic is false: skip the rest of this line AND all attached lines (indented or starting with &)
                    let base_line = tokens[*i - 1].line;
                    while *i < tokens.len() && tokens[*i].line == base_line {
                        *i += 1;
                    }

                    loop {
                        if *i >= tokens.len() {
                            break;
                        }
                        let is_indented = tokens[*i].column > current_indent;
                        let is_ampersand = tokens[*i].token_type == TokenType::Ampersand;

                        if is_indented || is_ampersand {
                            let line = tokens[*i].line;
                            while *i < tokens.len() && tokens[*i].line == line {
                                *i += 1;
                            }
                        } else {
                            break;
                        }
                    }
                    *i -= 1; // Main loop will increment
                }
            }
            TokenType::Or => {
                *i += 1;
                if self.last_condition_met {
                    // Skip the 'or' block
                    while *i < tokens.len() && tokens[*i].column > current_indent {
                        let line = tokens[*i].line;
                        while *i < tokens.len() && tokens[*i].line == line {
                            *i += 1;
                        }
                    }
                    *i -= 1;
                } else {
                    // Previous IF failed, run this one
                    self.last_condition_met = true;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::If {
                        *i -= 1;
                    } else {
                        *i -= 1;
                    }
                }
            }
            TokenType::Start | TokenType::Sloop => {
                let start_idx = *i;
                let is_sloop = token_type == TokenType::Sloop;
                *i += 1;
                if *i < tokens.len() || is_sloop {
                    if !is_sloop && tokens[*i].token_type == TokenType::Doing {
                        *i += 1;
                        let func_name = self.get_token_raw_name(&tokens[*i]);
                        if !func_name.is_empty() {
                            let name = func_name;
                            let mut params = Vec::new();
                            *i += 1;

                            // Parse parameters (x, y, z)
                            if *i < tokens.len() && tokens[*i].token_type == TokenType::LParen {
                                *i += 1;
                                while *i < tokens.len()
                                    && tokens[*i].token_type != TokenType::RParen
                                {
                                    if let TokenType::Identifier(param) = &tokens[*i].token_type {
                                        params.push(param.clone());
                                    }
                                    *i += 1;
                                    if *i < tokens.len()
                                        && tokens[*i].token_type == TokenType::Char(',')
                                    {
                                        *i += 1;
                                    }
                                }
                                if *i < tokens.len() && tokens[*i].token_type == TokenType::RParen {
                                    *i += 1;
                                }
                            }

                            let mut func_tokens = Vec::new();

                            while *i < tokens.len() {
                                if tokens[*i].token_type == TokenType::End
                                    && *i + 1 < tokens.len()
                                    && tokens[*i + 1].token_type == TokenType::Doing
                                {
                                    *i += 1;
                                    break;
                                }
                                func_tokens.push(tokens[*i].clone());
                                *i += 1;
                            }
                            if let Ok(mut funcs) = self.functions.write() {
                                funcs.insert(name, (params, func_tokens));
                            }
                        }
                    } else if !is_sloop && tokens[*i].token_type == TokenType::View {
                        *i += 1;
                        if let TokenType::Identifier(view_name) = &tokens[*i].token_type {
                            let name = view_name.clone();
                            let mut view_tokens = Vec::new();
                            *i += 1;
                            while *i < tokens.len() {
                                if tokens[*i].token_type == TokenType::End
                                    && *i + 1 < tokens.len()
                                    && tokens[*i + 1].token_type == TokenType::View
                                {
                                    *i += 1;
                                    break;
                                }
                                view_tokens.push(tokens[*i].clone());
                                *i += 1;
                            }
                            if let Ok(mut views) = self.views.write() {
                                println!(
                                    "ENGINE: Registered View '{}' with {} tokens",
                                    name,
                                    view_tokens.len()
                                );
                                views.insert(name, view_tokens);
                            }
                        }
                    } else if !is_sloop && tokens[*i].token_type == TokenType::Style {
                        *i += 1;

                        // Capture the full name (including dots)
                        let mut name = String::new();
                        while *i < tokens.len()
                            && !matches!(tokens[*i].token_type, TokenType::Plus | TokenType::EOF)
                            && tokens[*i].line == tokens[*i - 1].line
                        {
                            name.push_str(&self.get_token_value(&tokens[*i]));
                            *i += 1;
                        }

                        if !name.is_empty() {
                            let mut properties = HashMap::new();
                            while *i < tokens.len() {
                                if tokens[*i].token_type == TokenType::End
                                    && *i + 1 < tokens.len()
                                    && tokens[*i + 1].token_type == TokenType::Style
                                {
                                    *i += 1;
                                    break;
                                }

                                let prop_name = match &tokens[*i].token_type {
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
                                    *i += 1;
                                    let val = self.get_complex_value(i, tokens);
                                    properties.insert(prop, val);
                                    *i -= 1; // Adjust for outer increment
                                }
                                *i += 1;
                            }
                            if let Ok(mut styles) = self.styles.write() {
                                styles.insert(name.to_lowercase(), properties);
                            }
                        }
                    } else if is_sloop || tokens[*i].token_type == TokenType::Loop {
                        let mut limit = 0;

                        // Check for .r.N syntax
                        if *i + 4 < tokens.len()
                            && tokens[*i + 1].token_type == TokenType::Dot
                            && self.get_token_value(&tokens[*i + 2]) == "r"
                            && tokens[*i + 3].token_type == TokenType::Dot
                        {
                            if let TokenType::Number(n) = tokens[*i + 4].token_type {
                                limit = n as usize;
                                *i += 4;
                            }
                        }

                        // Check for For-Each: loop on {List/Map} as {Var} [ValVar]
                        let mut is_foreach = false;
                        let mut foreach_list = Vec::new();
                        let mut foreach_keys = Vec::new();
                        let mut foreach_var = String::new();
                        let mut foreach_val_var = String::new();
                        let mut map_json = String::new(); // Store raw map for lookups

                        let mut check_idx = *i;
                        if !is_sloop {
                            check_idx += 1;
                        } // Skip 'loop' if not using sloop

                        if check_idx < tokens.len()
                            && (tokens[check_idx].token_type == TokenType::On
                                || tokens[check_idx].token_type == TokenType::At)
                        {
                            *i = check_idx + 1; // skip Loop/Sloop and On/At

                            // Get Data Source (Usually {List} or {Map})
                            if *i < tokens.len() && tokens[*i].token_type == TokenType::LBrace {
                                *i += 1;
                                if let TokenType::Identifier(source_name) = &tokens[*i].token_type {
                                    let raw_data = self
                                        .get_variable(source_name)
                                        .unwrap_or(String::from("[]"));
                                    if raw_data.trim().starts_with('{') {
                                        // MAP
                                        foreach_keys = self.parse_json_map_keys(&raw_data);
                                        // println!("DEBUG START: Found Map Keys: {:?}", foreach_keys);
                                        map_json = raw_data;
                                    } else {
                                        // LIST
                                        foreach_list = self.parse_json_list(&raw_data);
                                    }
                                    *i += 2; // skip Name and }
                                }
                            }

                            // Get 'as' {Var} [ValVar]
                            if *i < tokens.len() && tokens[*i].token_type == TokenType::As {
                                *i += 1;
                                // 1st Variable (Item or Key)
                                if *i < tokens.len() && tokens[*i].token_type == TokenType::LBrace {
                                    *i += 1;

                                    // Allow ANY token as variable name (e.g. {File})
                                    let var_name = self.get_token_raw_name(&tokens[*i]);

                                    if !var_name.is_empty() {
                                        foreach_var = var_name;
                                        *i += 2; // skip Name and }
                                        is_foreach = true;
                                    }
                                }
                                // Optional 2nd Variable (Value for Maps)
                                if *i < tokens.len() && tokens[*i].token_type == TokenType::LBrace {
                                    *i += 1;
                                    let val_name = self.get_token_raw_name(&tokens[*i]);
                                    if !val_name.is_empty() {
                                        foreach_val_var = val_name;
                                        *i += 2; // skip Name and }
                                    }
                                }

                                // Initialize First Item
                                if is_foreach {
                                    if !foreach_list.is_empty() {
                                        self.set_variable(
                                            foreach_var.clone(),
                                            foreach_list[0].clone(),
                                        );
                                    } else if !foreach_keys.is_empty() {
                                        let key = foreach_keys[0].clone();
                                        self.set_variable(foreach_var.clone(), key.clone());
                                        if !foreach_val_var.is_empty() {
                                            let val = self.get_map_value(&map_json, &key);
                                            self.set_variable(foreach_val_var.clone(), val);
                                        }
                                    }
                                }
                            }
                            *i -= 1; // Adjust for outer increment
                        }

                        if !is_sloop {
                            *i += 1;
                        }
                        let mut valid = false;
                        if *i < tokens.len() && tokens[*i].column > tokens[start_idx].column {
                            valid = true;
                        }
                        self.loop_stack.push(LoopInfo {
                            start_index: *i,
                            is_valid: valid,
                            limit,
                            count: 0,
                            is_foreach,
                            foreach_list,
                            foreach_keys,
                            foreach_var,
                            foreach_val_var,
                            foreach_source_json: map_json,
                            foreach_idx: 0,
                        });
                        *i -= 1; // Back up so main loop increments into the first loop token
                    }
                }
            }
            TokenType::Keep => {
                *i += 1;
                if let Some(mut loop_info) = self.loop_stack.pop() {
                    if loop_info.is_valid {
                        if loop_info.is_foreach {
                            loop_info.foreach_idx += 1;

                            let mut has_next = false;
                            if !loop_info.foreach_list.is_empty() {
                                if loop_info.foreach_idx < loop_info.foreach_list.len() {
                                    let item =
                                        loop_info.foreach_list[loop_info.foreach_idx].clone();
                                    self.set_variable(loop_info.foreach_var.clone(), item);
                                    has_next = true;
                                }
                            } else if !loop_info.foreach_keys.is_empty()
                                && loop_info.foreach_idx < loop_info.foreach_keys.len() {
                                    let key = loop_info.foreach_keys[loop_info.foreach_idx].clone();
                                    self.set_variable(loop_info.foreach_var.clone(), key.clone());

                                    if !loop_info.foreach_val_var.is_empty() {
                                        let val = self
                                            .get_map_value(&loop_info.foreach_source_json, &key);
                                        self.set_variable(loop_info.foreach_val_var.clone(), val);
                                    }
                                    has_next = true;
                                }

                            if has_next {
                                *i = loop_info.start_index;
                                *i -= 1;
                                self.loop_stack.push(loop_info);
                            }
                            return;
                        }

                        if loop_info.limit > 0 && loop_info.count + 1 >= loop_info.limit {
                            // Safety Limit Reached
                            return;
                        }
                        loop_info.count += 1;
                        let mut condition_tokens = Vec::new();
                        let line = tokens[*i].line;
                        let mut j = *i;
                        while j < tokens.len() && tokens[j].line == line {
                            condition_tokens.push(tokens[j].clone());
                            j += 1;
                        }

                        // Special: "keep loop" (Refuel)
                        let mut refuel = false;
                        if condition_tokens.len() == 1
                            && self.get_token_value(&condition_tokens[0]) == "loop"
                        {
                            refuel = true;
                        }

                        if refuel {
                            loop_info.count = 0; // Reset fuel
                            *i = loop_info.start_index;
                            *i -= 1;
                            self.loop_stack.push(loop_info);
                        } else if self.evaluate_condition(&condition_tokens) {
                            *i = loop_info.start_index;
                            *i -= 1;
                            self.loop_stack.push(loop_info);
                        } else {
                            *i = j - 1;
                        }
                    }
                }
            }
            TokenType::Stop | TokenType::Loops => {
                let is_loops = token_type == TokenType::Loops;
                *i += 1;
                if *i < tokens.len() || is_loops {
                    if is_loops || tokens[*i].token_type == TokenType::Loop {
                        if !self.loop_stack.is_empty() {
                            self.loop_stack.pop();
                            let mut depth = 1;
                            while *i < tokens.len() && depth > 0 {
                                if tokens[*i].token_type == TokenType::Start
                                    && (*i + 1 < tokens.len()
                                        && tokens[*i + 1].token_type == TokenType::Loop)
                                {
                                    depth += 1;
                                }
                                if tokens[*i].token_type == TokenType::Sloop {
                                    depth += 1;
                                }
                                if tokens[*i].token_type == TokenType::Keep {
                                    depth -= 1;
                                    if depth == 0 {
                                        let line = tokens[*i].line;
                                        while *i < tokens.len() && tokens[*i].line == line {
                                            *i += 1;
                                        }
                                        *i -= 1;
                                        break;
                                    }
                                }
                                *i += 1;
                            }
                        }
                    } else if tokens[*i].token_type == TokenType::Run {
                        std::process::exit(0);
                    }
                }
            }
            TokenType::Addrun => {
                *i += 1;
                if *i < tokens.len() {
                    match &tokens[*i].token_type {
                        TokenType::End => std::process::exit(0),
                        TokenType::Identifier(name) => {
                            let func_data = {
                                if let Ok(funcs) = self.functions.read() {
                                    funcs.get(name).cloned()
                                } else {
                                    None
                                }
                            };
                            if let Some((_, func_tokens)) = func_data {
                                self.run(func_tokens);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_step(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1; // consume step

        // step {Var}
        if *i >= tokens.len() {
            return;
        }
        let var_name = self.get_token_raw_name(&tokens[*i]);
        *i += 1;

        // towards {Target}
        if *i >= tokens.len() || tokens[*i].token_type != TokenType::Towards {
            return;
        }
        *i += 1;
        let target = self.consume_math(i, tokens);

        // speed {Nx}
        let mut speed = 1.0;
        if *i < tokens.len() && tokens[*i].token_type == TokenType::Speed {
            *i += 1;
            if *i < tokens.len() {
                match tokens[*i].token_type {
                    TokenType::SpeedVal(val) => {
                        speed = val;
                        *i += 1;
                    }
                    _ => {
                        // STRICT RULE: If not a SpeedVal (Nx), it's invalid.
                        // We do not accept raw numbers.
                        println!("BigC Error: Speed must use 'Nx' syntax (e.g. 0.5x).");
                        return;
                    }
                }
            }
        }

        // Perform Smooth Damping (Frame Rate Independent)
        // Formula: current = lerp(current, target, 1.0 - exp(-decay * dt))
        // 'speed' acts as the decay constant.
        // Higher speed = faster convergence.
        // 0.5x -> speed = 0.5.
        // 5.0x -> speed = 5.0.

        let current_str = self.get_variable(&var_name).unwrap_or(String::from("0"));
        let current = current_str.parse::<f32>().unwrap_or(0.0);

        // Calculate DT (Seconds since last frame)
        // If GUI is enabled, use the stable_dt provided by the engine
        let dt = if self.guy_enabled {
            if self.current_dt > 0.1 {
                0.1
            } else {
                self.current_dt
            }
        } else {
            let raw_dt = self.last_delta_tick.elapsed().as_secs_f32();
            if raw_dt > 0.1 {
                0.1
            } else {
                raw_dt
            }
        };

        // Use exponential decay for smooth, frame-independent movement
        // speed is the "lambda" (decay constant).
        // If speed is 1.0, it closes ~63% of the gap in 1 second.
        // If speed is 5.0, it closes ~99% of the gap in 1 second.

        // We multiply speed by 5.0 to make "1x" feel snappy (closing gap in ~0.2s)
        // Otherwise 1x is too slow/floaty.
        let adjusted_speed = speed as f32 * 5.0;

        let t = 1.0 - (-adjusted_speed * dt).exp();

        let diff = target - current;
        let new_val = current + (diff * t);

        // Snap if close (0.5 pixel)
        if diff.abs() < 0.5 {
            self.set_variable(var_name, target.to_string());
        } else {
            self.set_variable(var_name, new_val.to_string());
        }
    }
}
