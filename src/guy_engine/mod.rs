pub mod elements;
pub mod window;

use eframe::egui;
pub use window::BigGuyApp;

use crate::interpreter::Interpreter;
use crate::tokens::Token;
use cosmic_text::{FontSystem, SwashCache};
use std::sync::{Arc, Mutex};

pub fn run_direct(interpreter: &mut Interpreter, filename: &str, tokens: Vec<Token>) {
    println!("BigGuy: Direct Launch '{}'", filename);

    // 1. Run the script once to register styles and named views
    let was_enabled = interpreter.guy_enabled;
    interpreter.guy_enabled = false;
    interpreter.run(tokens.clone());
    interpreter.guy_enabled = was_enabled;

    // 2. Identify which view to launch.
    let has_main = if let Ok(v) = interpreter.views.read() {
        v.contains_key("Main")
    } else {
        false
    };
    if !has_main {
        if let Ok(mut v) = interpreter.views.write() {
            v.insert(String::from("Main"), tokens);
        }
    }

    // Initialize Resources
    let mut font_system = FontSystem::new();
    if let Ok(dir) = std::fs::read_dir("fonts") {
        for entry in dir.flatten() {
            font_system.db_mut().load_font_file(entry.path()).ok();
        }
    }
    let font_system = Arc::new(Mutex::new(font_system));
    let swash_cache = Arc::new(Mutex::new(SwashCache::new()));

    // Clone Interpreter state
    let interpreter_clone = interpreter.clone();

    // Launch eframe
    eframe::run_native(
        "BigGuy Direct",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            let mut fonts = egui::FontDefinitions::default();
            if let Ok(dir) = std::fs::read_dir("fonts") {
                for entry in dir.flatten() {
                    if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                        if name.to_lowercase().contains("padauk") {
                            continue;
                        }
                        if let Ok(bytes) = std::fs::read(entry.path()) {
                            let font_name = name.to_string();
                            fonts
                                .font_data
                                .insert(font_name.clone(), egui::FontData::from_owned(bytes));

                            // Register as its own name (This is the critical fix)
                            fonts.families.insert(
                                egui::FontFamily::Name(font_name.clone().into()),
                                vec![font_name.clone()],
                            );

                            let lower_name = font_name.to_lowercase();
                            let is_variant =
                                lower_name.contains("bold") || lower_name.contains("italic");
                            let generic = if lower_name.contains("mono") {
                                egui::FontFamily::Monospace
                            } else {
                                egui::FontFamily::Proportional
                            };

                            if is_variant {
                                fonts.families.get_mut(&generic).unwrap().push(font_name);
                            } else {
                                fonts
                                    .families
                                    .get_mut(&generic)
                                    .unwrap()
                                    .insert(0, font_name);
                            }
                        }
                    }
                }
            }
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cc.egui_ctx.set_fonts(fonts);
            }));

            Ok(Box::new(BigGuyApp::new(
                interpreter_clone,
                String::from("Main"),
                font_system,
                swash_cache,
            )))
        }),
    )
    .ok();
}

pub fn handle_start(interpreter: &mut Interpreter, i: &mut usize, tokens: &Vec<Token>) {
    if !interpreter.guy_enabled {
        return;
    }
    *i += 2; // Skip start guy

    let initial_view = if *i < tokens.len() {
        interpreter.get_token_value(&tokens[*i])
    } else {
        String::new()
    };

    // Safety: If already in a GUI loop, don't start another one
    if interpreter
        .variables
        .read()
        .unwrap()
        .contains_key("InGuyLoop")
    {
        return;
    }
    interpreter.set_variable(String::from("InGuyLoop"), String::from("true"));

    let mut font_system = FontSystem::new();
    if let Ok(dir) = std::fs::read_dir("fonts") {
        for entry in dir.flatten() {
            font_system.db_mut().load_font_file(entry.path()).ok();
        }
    }
    let font_system = Arc::new(Mutex::new(font_system));
    let swash_cache = Arc::new(Mutex::new(SwashCache::new()));

    let interpreter_clone = interpreter.clone();
    eframe::run_native(
        "BigC - BigGuy",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            let mut fonts = egui::FontDefinitions::default();
            if let Ok(dir) = std::fs::read_dir("fonts") {
                for entry in dir.flatten() {
                    if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                        if name.to_lowercase().contains("padauk") {
                            continue;
                        }
                        if let Ok(bytes) = std::fs::read(entry.path()) {
                            let font_name = name.to_string();
                            fonts
                                .font_data
                                .insert(font_name.clone(), egui::FontData::from_owned(bytes));
                            fonts.families.insert(
                                egui::FontFamily::Name(font_name.clone().into()),
                                vec![font_name.clone()],
                            );

                            let lower_name = font_name.to_lowercase();
                            let is_variant =
                                lower_name.contains("bold") || lower_name.contains("italic");
                            let generic = if lower_name.contains("mono") {
                                egui::FontFamily::Monospace
                            } else {
                                egui::FontFamily::Proportional
                            };
                            if is_variant {
                                fonts.families.get_mut(&generic).unwrap().push(font_name);
                            } else {
                                fonts
                                    .families
                                    .get_mut(&generic)
                                    .unwrap()
                                    .insert(0, font_name);
                            }
                        }
                    }
                }
            }
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cc.egui_ctx.set_fonts(fonts);
            }));
            Ok(Box::new(BigGuyApp::new(
                interpreter_clone,
                initial_view,
                font_system,
                swash_cache,
            )))
        }),
    )
    .ok();
}
