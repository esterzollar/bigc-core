use crate::interpreter::Interpreter;
use crate::tokens::Token;

pub struct BigDebug;

impl BigDebug {
    // --- Trace Mode ---
    // Prints the current line of code before it executes.
    // This helps users see exactly where the engine is.
    pub fn trace(interpreter: &Interpreter, token: &Token) {
        // Only trace if --debug flag is active
        // Note: We need a way to check the flag.
        // For now, we will check a global variable "BigDebug" which we will set in main.rs
        if let Some(val) = interpreter.get_variable("BigDebug") {
            if val == "true" || val == "1" {
                let lines: Vec<&str> = interpreter.full_source.lines().collect();
                if token.line > 0 && token.line <= lines.len() {
                    let line_content = lines[token.line - 1].trim();
                    // Avoid spamming trace for every token on the line.
                    // Only print when we hit the first token of a new line or a significant instruction.
                    // A simple heuristic: If this token is the first one we processed on this line in this loop step.
                    // But interpreter.last_line_pos is updated *after* processing usually.
                    // Let's just print it. The user asked for "Trace Mode".

                    println!("[TRACE] Line {}: {}", token.line, line_content);
                }
            }
        }
    }

    // --- Variable Watch ---
    // Logs when a variable is updated.
    pub fn log_var_change(name: &str, old_val: Option<&String>, new_val: &str, is_debug: bool) {
        if is_debug {
            match old_val {
                Some(old) => {
                    if old != new_val {
                        println!("[DEBUG] {} changed: '{}' -> '{}'", name, old, new_val);
                    }
                }
                None => {
                    println!("[DEBUG] {} created: '{}'", name, new_val);
                }
            }
        }
    }
}
