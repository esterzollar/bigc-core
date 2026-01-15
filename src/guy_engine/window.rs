// BigC Window & Rendering Loop
// [BigC Universal Public Mandate License (BUPML): Open for Contribution]

use crate::interpreter::Interpreter;
use crate::tokens::{Token, TokenType};
use cosmic_text::{
    Attrs, Buffer, Color as CosmicColor, Family, FontSystem, Metrics, Shaping, SwashCache, Weight,
};
use eframe::{egui, App};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

pub struct BigGuyApp {
    pub(crate) interpreter: Interpreter,
    pub(crate) current_view: String,
    pub(crate) font_system: Arc<Mutex<FontSystem>>,
    pub(crate) swash_cache: Arc<Mutex<SwashCache>>,
    pub(crate) texture_cache: HashMap<String, egui::TextureHandle>,
    pub(crate) texture_usage: VecDeque<String>,
    pub(crate) cache_limit: usize,
    pub(crate) frame_count: u64,
}

impl App for BigGuyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.interpreter.current_dt = ctx.input(|i| i.stable_dt);

        // --- NERVOUS SYSTEM (Input) ---
        let screen_rect = ctx.screen_rect();
        self.interpreter
            .set_variable("WindowWidth".to_string(), screen_rect.width().to_string());
        self.interpreter
            .set_variable("WindowHeight".to_string(), screen_rect.height().to_string());

        if let Some(pos) = ctx.pointer_latest_pos() {
            if let Ok(mut m) = self.interpreter.mouse_pos.write() {
                *m = (pos.x, pos.y);
            }
        }
        ctx.input(|i| {
            if let Ok(mut keys) = self.interpreter.keys_down.write() {
                keys.clear();
                for key in &i.keys_down {
                    let name = format!("{:?}", key).to_lowercase();
                    keys.insert(name, true);
                }
            }
            if let Ok(mut clicked) = self.interpreter.global_clicked.write() {
                *clicked = i.pointer.primary_clicked();
                if *clicked {
                    println!("BigGuy DEBUG: GLOBAL CLICK DETECTED!");
                }
            }
        });

        if self.frame_count == 0 {
            println!(
                "BigGuy DEBUG: Frame 0. Running Init for view '{}'",
                self.current_view
            );
            self.run_init(ctx);
        }
        self.frame_count += 1;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 20)))
            .show(ctx, |ui| {
                if let Ok(mut clicked) = self.interpreter.last_widget_clicked.write() {
                    *clicked = false;
                }

                if let Ok(mut current) = self.interpreter.clicked_tags.write() {
                    if let Ok(mut last) = self.interpreter.last_clicked_tags.write() {
                        *last = current.clone();
                    }
                    current.clear();
                }

                // STATE SWAP: Current -> Last, then Clear Current
                if let Ok(mut current) = self.interpreter.hovered_tags.write() {
                    if let Ok(mut last) = self.interpreter.last_hovered_tags.write() {
                        *last = current.clone();
                    }
                    current.clear();
                }
                if let Ok(mut current) = self.interpreter.pressed_tags.write() {
                    if let Ok(mut last) = self.interpreter.last_pressed_tags.write() {
                        *last = current.clone();
                    }
                    current.clear();
                }
                if let Ok(mut current) = self.interpreter.dragged_tags.write() {
                    if let Ok(mut last) = self.interpreter.last_dragged_tags.write() {
                        *last = current.clone();
                    }
                    current.clear();
                }
                if let Ok(mut current) = self.interpreter.frame_drag_delta.write() {
                    if let Ok(mut last) = self.interpreter.last_drag_delta.write() {
                        *last = *current;
                    }
                    *current = (0.0, 0.0);
                }

                self.render(ui);
            });

        ctx.request_repaint();
    }
}

impl BigGuyApp {
    fn run_init(&mut self, ctx: &egui::Context) {
        let tokens_opt = if let Ok(views) = self.interpreter.views.read() {
            views.get(&self.current_view).cloned()
        } else {
            None
        };
        if let Some(tokens) = tokens_opt {
            let mut i = 0;
            while i < tokens.len() {
                if tokens[i].token_type == TokenType::Init {
                    i += 1;
                    while i < tokens.len() {
                        match tokens[i].token_type {
                            TokenType::End => {
                                if i + 1 < tokens.len()
                                    && tokens[i + 1].token_type == TokenType::Doing
                                {
                                    i += 1;
                                }
                                return;
                            }
                            TokenType::Identifier(ref name) => {
                                if i + 1 < tokens.len()
                                    && tokens[i + 1].token_type == TokenType::Assign
                                {
                                    self.interpreter.handle_assignment(
                                        &mut i,
                                        &tokens,
                                        name.clone(),
                                    );
                                } else {
                                    i += 1;
                                }
                            }
                            TokenType::Run => self.handle_run(&mut i, &tokens),
                            TokenType::Set => {
                                i += 1;
                                if i < tokens.len() && tokens[i].token_type == TokenType::Window {
                                    i += 1;
                                    match &tokens[i].token_type {
                                        TokenType::Title => {
                                            i += 1;
                                            let t =
                                                self.interpreter.get_complex_value(&mut i, &tokens);
                                            ctx.send_viewport_cmd(egui::ViewportCommand::Title(t));
                                            i -= 1;
                                        }
                                        TokenType::Identifier(s) if s == "size" => {
                                            i += 1;
                                            let w = self.interpreter.consume_math(&mut i, &tokens);
                                            let h = self.interpreter.consume_math(&mut i, &tokens);
                                            ctx.send_viewport_cmd(
                                                egui::ViewportCommand::InnerSize(egui::vec2(w, h)),
                                            );
                                            i -= 1;
                                        }
                                        TokenType::Identifier(s) if s == "fullscreen" => {
                                            i += 1;
                                            let val =
                                                self.interpreter.get_complex_value(&mut i, &tokens);
                                            ctx.send_viewport_cmd(
                                                egui::ViewportCommand::Fullscreen(val == "true"),
                                            );
                                            i -= 1;
                                        }
                                        _ => {}
                                    }
                                } else if i < tokens.len() {
                                    if let TokenType::Identifier(ref name) =
                                        tokens[i].token_type.clone()
                                    {
                                        self.interpreter.handle_assignment(
                                            &mut i,
                                            &tokens,
                                            name.clone(),
                                        );
                                    }
                                }
                            }
                            _ => i += 1,
                        }
                    }
                }
                i += 1;
            }
        }
    }

    pub(crate) fn new(
        interpreter: Interpreter,
        view_name: String,
        font_system: Arc<Mutex<FontSystem>>,
        swash_cache: Arc<Mutex<SwashCache>>,
    ) -> Self {
        Self {
            interpreter,
            current_view: view_name,
            font_system,
            swash_cache,
            texture_cache: HashMap::new(),
            texture_usage: VecDeque::new(),
            cache_limit: 1000,
            frame_count: 0,
        }
    }

    fn update_cache_usage(&mut self, key: &str) {
        if let Some(pos) = self.texture_usage.iter().position(|k| k == key) {
            self.texture_usage.remove(pos);
        }
        self.texture_usage.push_back(key.to_string());
    }

    fn evict_if_full(&mut self) {
        while self.texture_usage.len() > self.cache_limit {
            if let Some(old_key) = self.texture_usage.pop_front() {
                self.texture_cache.remove(&old_key);
            }
        }
    }

    pub(crate) fn get_cached_image_file(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> Option<egui::TextureHandle> {
        if let Some(handle) = self.texture_cache.get(path).cloned() {
            self.update_cache_usage(path);
            return Some(handle);
        }

        if let Some(bytes) = self.interpreter.resolve_file(path) {
            if let Ok(image) = image::load_from_memory(&bytes) {
                let size = [image.width() as _, image.height() as _];
                let handle = ctx.load_texture(
                    path,
                    egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        image.to_rgba8().as_flat_samples().as_slice(),
                    ),
                    egui::TextureOptions::LINEAR,
                );
                self.texture_cache.insert(path.to_string(), handle.clone());
                self.update_cache_usage(path);
                self.evict_if_full();
                return Some(handle);
            }
        }
        None
    }

    pub(crate) fn get_cached_texture(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        text: &str,
        size: f32,
        color: egui::Color32,
        font_name: &str,
        line_height: f32,
    ) -> Option<egui::TextureHandle> {
        if let Some(handle) = self.texture_cache.get(key).cloned() {
            self.update_cache_usage(key);
            return Some(handle);
        }

        let width_height_pixels = {
            let mut font_system = self.font_system.lock().unwrap();
            let mut swash_cache = self.swash_cache.lock().unwrap();
            let metrics = Metrics::new(size, size * line_height);
            let mut buffer = Buffer::new(&mut font_system, metrics);
            let mut attrs = Attrs::new()
                .color(CosmicColor::rgba(
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a(),
                ))
                .weight(Weight::NORMAL);
            if !font_name.is_empty() {
                attrs = attrs.family(Family::Name(font_name));
            }
            buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced, None);
            buffer.shape_until_scroll(&mut font_system, false);
            let mut min_x = i32::MAX;
            let mut min_y = i32::MAX;
            let mut max_x = i32::MIN;
            let mut max_y = i32::MIN;
            for run in buffer.layout_runs() {
                for glyph in run.glyphs {
                    let physical = glyph.physical((0.0, 0.0), 1.0);
                    if let Some(image) = swash_cache.get_image(&mut font_system, physical.cache_key)
                    {
                        let x = (glyph.x + image.placement.left as f32).floor() as i32;
                        let y = (run.line_y + glyph.y - image.placement.top as f32).floor() as i32;
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x + image.placement.width as i32);
                        max_y = max_y.max(y + image.placement.height as i32);
                    }
                }
            }
            if min_x >= max_x {
                return None;
            }
            let width = (max_x - min_x) as usize + 10;
            let height = (max_y - min_y) as usize + 10;
            let mut pixels = vec![0u8; width * height * 4];
            for run in buffer.layout_runs() {
                for glyph in run.glyphs {
                    let physical = glyph.physical((0.0, 0.0), 1.0);
                    if let Some(image) = swash_cache.get_image(&mut font_system, physical.cache_key)
                    {
                        let x = (glyph.x + image.placement.left as f32).floor() as i32 - min_x;
                        let y = (run.line_y + glyph.y - image.placement.top as f32).floor() as i32
                            - min_y;
                        for row in 0..image.placement.height as usize {
                            for col in 0..image.placement.width as usize {
                                let tx = x + col as i32;
                                let ty = y + row as i32;
                                if tx >= 0 && tx < width as i32 && ty >= 0 && ty < height as i32 {
                                    let idx = (ty as usize * width + tx as usize) * 4;
                                    let val =
                                        image.data[row * image.placement.width as usize + col];
                                    if val > 0 {
                                        pixels[idx] = color.r();
                                        pixels[idx + 1] = color.g();
                                        pixels[idx + 2] = color.b();
                                        pixels[idx + 3] = val;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            (width, height, pixels)
        };

        let (width, height, pixels) = width_height_pixels;
        let handle = ctx.load_texture(
            key,
            egui::ColorImage::from_rgba_unmultiplied([width, height], &pixels),
            egui::TextureOptions::LINEAR,
        );
        self.texture_cache.insert(key.to_string(), handle.clone());
        self.update_cache_usage(key);
        self.evict_if_full();
        Some(handle)
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        let tokens_opt = if let Ok(views) = self.interpreter.views.read() {
            views.get(&self.current_view).cloned()
        } else {
            None
        };
        if let Some(tokens) = tokens_opt {
            let mut i = 0;
            self.render_recursive(ui, &tokens, &mut i);
        } else {
            println!("BigGuy DEBUG: View '{}' NOT FOUND!", self.current_view);
        }
    }

    pub(crate) fn render_recursive(
        &mut self,
        ui: &mut egui::Ui,
        tokens: &Vec<Token>,
        i: &mut usize,
    ) {
        while *i < tokens.len() {
            match tokens[*i].token_type {
                TokenType::Draw => {
                    *i += 1;
                    self.handle_draw(i, tokens, ui);
                }
                TokenType::Step => self.interpreter.handle_step(i, tokens),
                TokenType::Print | TokenType::Play | TokenType::Beep | TokenType::Asset => {
                    self.interpreter.handle_action(i, tokens);
                    *i += 1;
                }
                TokenType::Stop => self.interpreter.handle_control(i, tokens),
                TokenType::Set => {
                    *i += 1;
                    if *i < tokens.len() && tokens[*i].token_type == TokenType::Window {
                        *i += 1;
                        match &tokens[*i].token_type {
                            TokenType::Title => {
                                *i += 1;
                                let t = self.interpreter.get_complex_value(i, tokens);
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title(t));
                                *i -= 1;
                            }
                            TokenType::Identifier(s) if s == "size" => {
                                *i += 1;
                                let w = self.interpreter.consume_math(i, tokens);
                                let h = self.interpreter.consume_math(i, tokens);
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(
                                    egui::vec2(w, h),
                                ));
                                *i -= 1;
                            }
                            TokenType::Identifier(s) if s == "fullscreen" => {
                                *i += 1;
                                let val = self.interpreter.get_complex_value(i, tokens);
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                        val == "true",
                                    ));
                                *i -= 1;
                            }
                            _ => {}
                        }
                    } else if *i < tokens.len() && tokens[*i].token_type == TokenType::Volume {
                        self.interpreter.handle_action(i, tokens);
                    } else if *i < tokens.len() {
                        if let TokenType::Identifier(ref name) = tokens[*i].token_type.clone() {
                            self.interpreter.handle_assignment(i, tokens, name.clone());
                        }
                    }
                }
                TokenType::Init => {
                    *i += 1;
                    while *i < tokens.len() {
                        if tokens[*i].token_type == TokenType::End {
                            *i += 1;
                            if *i < tokens.len() && tokens[*i].token_type == TokenType::Doing {
                                *i += 1;
                            }
                            break;
                        }
                        *i += 1;
                    }
                }
                TokenType::Start => {
                    if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::Guy {
                        *i += 2;
                    } else if *i + 1 < tokens.len()
                        && matches!(
                            tokens[*i + 1].token_type,
                            TokenType::Loop | TokenType::Sloop
                        )
                    {
                        println!("BigGuy Error: 'start loop' is forbidden in Views! Logic runs every frame.");
                        *i += 2;
                    } else {
                        self.interpreter.handle_control(i, tokens);
                    }
                }
                TokenType::Run => self.handle_run(i, tokens),
                TokenType::If => self.handle_if(i, tokens),
                TokenType::Or => {
                    let current_indent = tokens[*i].column;
                    if self.interpreter.last_condition_met {
                        let line = tokens[*i].line;
                        while *i < tokens.len() && tokens[*i].line == line {
                            *i += 1;
                        }
                        while *i < tokens.len() && tokens[*i].column > current_indent {
                            let skip_line = tokens[*i].line;
                            while *i < tokens.len() && tokens[*i].line == skip_line {
                                *i += 1;
                            }
                        }
                    } else {
                        *i += 1;
                        if *i < tokens.len() && tokens[*i].token_type == TokenType::If {
                            self.handle_if(i, tokens);
                        }
                    }
                }
                TokenType::Identifier(ref name) => {
                    if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::Assign {
                        self.interpreter.handle_assignment(i, tokens, name.clone());
                    } else {
                        *i += 1;
                    }
                }
                TokenType::End => {
                    if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::If {
                        *i += 2;
                    } else {
                        *i += 1;
                        return;
                    }
                }
                _ => *i += 1,
            }
        }
    }

    fn handle_run(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        *i += 1;
        if *i >= tokens.len() {
            return;
        }
        let name = self.interpreter.get_token_value(&tokens[*i]);
        let data = if let Ok(funcs) = self.interpreter.functions.read() {
            funcs.get(&name).cloned()
        } else {
            None
        };
        if let Some((_, func_tokens)) = data {
            let mut li = 0;
            self.render_recursive_standalone(&func_tokens, &mut li);
        }
        *i += 1;
    }

    pub(crate) fn render_recursive_standalone(&mut self, tokens: &Vec<Token>, i: &mut usize) {
        while *i < tokens.len() {
            match tokens[*i].token_type {
                TokenType::Identifier(ref name) => {
                    if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::Assign {
                        self.interpreter.handle_assignment(i, tokens, name.clone());
                    } else {
                        *i += 1;
                    }
                }
                TokenType::Step => self.interpreter.handle_step(i, tokens),
                TokenType::Start => self.interpreter.handle_control(i, tokens),
                TokenType::Print | TokenType::Play | TokenType::Beep | TokenType::Asset => {
                    self.interpreter.handle_action(i, tokens);
                    *i += 1;
                }
                TokenType::Or => {
                    let current_indent = tokens[*i].column;
                    if self.interpreter.last_condition_met {
                        let line = tokens[*i].line;
                        while *i < tokens.len() && tokens[*i].line == line {
                            *i += 1;
                        }
                        while *i < tokens.len() && tokens[*i].column > current_indent {
                            let skip_line = tokens[*i].line;
                            while *i < tokens.len() && tokens[*i].line == skip_line {
                                *i += 1;
                            }
                        }
                    } else {
                        *i += 1;
                        if *i < tokens.len() && tokens[*i].token_type == TokenType::If {
                            self.handle_if(i, tokens);
                        }
                    }
                }
                TokenType::Stop => self.interpreter.handle_control(i, tokens),
                TokenType::If => self.handle_if(i, tokens),
                TokenType::End => {
                    if *i + 1 < tokens.len() && tokens[*i + 1].token_type == TokenType::If {
                        *i += 2;
                    } else {
                        *i += 1;
                        return;
                    }
                }
                _ => *i += 1,
            }
        }
    }

    fn handle_if(&mut self, i: &mut usize, tokens: &Vec<Token>) {
        let current_indent = tokens[*i].column;
        let line = tokens[*i].line;
        *i += 1;
        let mut cond_tokens = Vec::new();
        while *i < tokens.len()
            && tokens[*i].token_type != TokenType::Ampersand
            && tokens[*i].line == line
        {
            cond_tokens.push(tokens[*i].clone());
            *i += 1;
        }
        let res = self.interpreter.evaluate_condition(&cond_tokens);
        self.interpreter.last_condition_met = res;
        if res {
            if *i < tokens.len() && tokens[*i].token_type == TokenType::Ampersand {
                *i += 1;
            }
        } else {
            // Skip rest of this line
            while *i < tokens.len() && tokens[*i].line == line {
                *i += 1;
            }

            // Skip all attached lines (indented or starting with &)
            loop {
                if *i >= tokens.len() {
                    break;
                }
                let is_indented = tokens[*i].column > current_indent;
                let is_ampersand = tokens[*i].token_type == TokenType::Ampersand;

                if is_indented || is_ampersand {
                    let skip_line = tokens[*i].line;
                    while *i < tokens.len() && tokens[*i].line == skip_line {
                        *i += 1;
                    }
                } else {
                    break;
                }
            }
        }
    }

    fn handle_draw(&mut self, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
        let t = tokens[*i].token_type.clone();
        match t {
            TokenType::Row => {
                *i += 1;
                let start = *i;
                let mut next_i = *i;
                ui.horizontal(|ui| {
                    let mut li = start;
                    self.render_recursive(ui, tokens, &mut li);
                    next_i = li;
                });
                *i = next_i;
            }
            TokenType::Column => {
                *i += 1;
                let start = *i;
                let mut next_i = *i;
                ui.vertical(|ui| {
                    let mut li = start;
                    self.render_recursive(ui, tokens, &mut li);
                    next_i = li;
                });
                *i = next_i;
            }
            TokenType::Text => super::elements::draw_text(self, i, tokens, ui),
            TokenType::Button => super::elements::draw_button(self, i, tokens, ui),
            TokenType::Input => super::elements::draw_input(self, i, tokens, ui),
            TokenType::Rectangle | TokenType::Rounded => {
                super::elements::draw_rect(self, i, tokens, ui)
            }
            TokenType::Circle => super::elements::draw_circle(self, i, tokens, ui),
            TokenType::Triangle => super::elements::draw_triangle(self, i, tokens, ui),
            TokenType::Line => super::elements::draw_line(self, i, tokens, ui),
            TokenType::Image => super::elements::draw_image(self, i, tokens, ui),
            TokenType::Markdown => super::elements::draw_markdown(self, i, tokens, ui),
            TokenType::Scroll => super::elements::draw_scroll_area(self, i, tokens, ui),
            _ => *i += 1,
        }
    }
}

pub fn css_color(s: &str) -> Result<egui::Color32, ()> {
    let s = s
        .trim_matches('"')
        .trim_matches('\'')
        .trim_start_matches('#');
    if let Ok(val) = u32::from_str_radix(s, 16) {
        Ok(egui::Color32::from_rgb(
            ((val >> 16) & 0xFF) as u8,
            ((val >> 8) & 0xFF) as u8,
            (val & 0xFF) as u8,
        ))
    } else {
        Ok(egui::Color32::WHITE)
    }
}
