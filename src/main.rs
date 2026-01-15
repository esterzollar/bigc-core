mod bigdebug;
mod bighelp;
mod bignet;
mod bigpack;
pub mod guy_engine;
mod interpreter;
mod lexer;
mod luck;
mod sound;
mod tokens;

use crate::bighelp::BigHelp;
use crate::bigpack::BigPack;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use std::env;
use std::fs;

fn print_usage() {
    println!("BigC Language Engine (bigrun) V.1.0 Mandate");
    println!("Usage: bigrun <file.big> [args]");
    println!("       bigrun whatis <keyword>");
    println!("       bigrun show <file.big>");
    println!("       bigrun pack <folder> <output.bigpak> [--key \"Secret\"]");
    println!("       bigrun bunpack <file.bigpak> [--key \"Secret\"]");
    println!("\nFlags:");
    println!("       -v, --version    Show engine version");
    println!("       -h, --help       Show this help message");
    println!("       --debug          Enable verbose trace and variable logging");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];

    if command == "--version" || command == "-v" {
        println!("BigC Language Engine (bigrun) V.1.0 Mandate");
        return;
    }

    if command == "--help" || command == "-h" {
        print_usage();
        return;
    }

    if command == "pack" {
        if args.len() < 4 {
            println!("Usage: bigrun pack <folder> <output.bigpak> [--key \"Secret\"]");
            return;
        }
        let key = if args.len() >= 6 && args[4] == "--key" {
            Some(args[5].clone())
        } else {
            None
        };
        BigPack::pack(&args[2], &args[3], key);
        return;
    }

    if command == "bunpack" {
        if args.len() < 3 {
            println!("Usage: bigrun bunpack <file.bigpak> [--key \"Secret\"]");
            return;
        }
        let key = if args.len() >= 5 && args[3] == "--key" {
            Some(args[4].clone())
        } else {
            None
        };
        BigPack::unpack(&args[2], key);
        return;
    }

    if command == "whatis" && args.len() >= 3 {
        BigHelp::whatis(&args[2]);
        return;
    }

    if command == "show" && args.len() >= 3 {
        BigHelp::show(&args[2]);
        return;
    }

    let filename = command;

    if !filename.ends_with(".big")
        && !filename.ends_with(".guy")
        && !filename.ends_with(".adkp")
        && !filename.ends_with(".bigpak")
    {
        println!("Big Error: I only speak .big, .guy, or .bigpak! Please provide a valid file.");
        return;
    }

    if filename.ends_with(".bigpak") {
        let mut interpreter = Interpreter::new();
        interpreter.mounted_archive = Some(filename.to_string());

        let entry_points = vec!["app.big", "main.big", "main.guy"];
        let mut entry_content = None;
        let mut entry_name = "";

        for ep in entry_points {
            if let Some(content) = interpreter.resolve_file_as_string(ep) {
                entry_content = Some(content);
                entry_name = ep;
                break;
            }
        }

        if let Some(content) = entry_content {
            println!("BigPack: Launching '{}' from archive...", entry_name);
            let mut lexer = Lexer::new(&content);
            let tokens = lexer.tokenize();

            if entry_name.ends_with(".guy") {
                interpreter.guy_enabled = true;
                if interpreter.sound_tx.is_none() {
                    interpreter.sound_tx = Some(crate::sound::start_sound_engine());
                }
                interpreter.run_guy_direct(entry_name, tokens);
            } else if interpreter.validate_syntax(&tokens) {
                interpreter.run(tokens);
            }
        } else {
            println!(
                "BigPack Error: No entry point found! (Expected app.big, main.big, or main.guy)"
            );
        }
        return;
    }

    let content = fs::read_to_string(filename).unwrap_or_else(|_| {
        println!("Big Error: Could not find or read file '{}'", filename);
        std::process::exit(1);
    });

    let mut lexer = Lexer::new(&content);
    let mut tokens = lexer.tokenize();

    let mut interpreter = Interpreter::new();
    interpreter.full_source = content.clone();

    if args.contains(&String::from("--debug")) {
        interpreter.set_variable(String::from("BigDebug"), String::from("true"));
        println!("BigC: Debug Mode Enabled. Tracing execution...");
    }

    if !std::path::Path::new("env_lib").exists() {
        let _ = fs::create_dir("env_lib");
    }

    if content.contains("attach fixer") {
        tokens = interpreter.heal_tokens(tokens);
    }

    if filename.ends_with(".guy") {
        interpreter.guy_enabled = true;
        if interpreter.sound_tx.is_none() {
            interpreter.sound_tx = Some(crate::sound::start_sound_engine());
        }
        interpreter.run_guy_direct(filename, tokens);
        return;
    }

    if interpreter.validate_syntax(&tokens) {
        interpreter.run(tokens);
    }

    if interpreter.last_error_pos.is_some() {
        std::process::exit(1);
    }
}
