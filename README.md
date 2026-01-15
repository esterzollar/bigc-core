# BigC Core Engine (Source Code)

[![License](https://img.shields.io/badge/License-BUPML-green)](LICENSE)
[![Foundation](https://img.shields.io/badge/Foundation-Rust-orange)](https://www.rust-lang.org/)

This is the official source code for the **BigC V.1.0 Mandate** engine (`bigrun`).

**BigC** is a human-first programming language designed to eliminate indentation hell and provide a linear, textbook-style coding experience. This engine is built in **Rust** for maximum performance, safety, and cross-platform compatibility.

---

## Architecture Overview

The engine is built as a single-pass tree-walk interpreter with the following components:

*   **`src/main.rs`**: The entry point. Handles CLI arguments, file reading, and the `BigPack` archive system.
*   **`src/lexer.rs`**: A custom, zero-copy tokenizer that converts raw text into `Token` streams.
*   **`src/interpreter/`**: The heart of the engine.
    *   **`mod.rs`**: The main evaluation loop (`Interpreter::run`).
    *   **`actions.rs`**: Handles I/O commands (`print`, `wait`, `ask`).
    *   **`control.rs`**: Manages flow control (`if`, `loop`, `doing`).
    *   **`guy_engine/`**: The BigGuy UI system integration (using `eframe`/`egui`).
    *   **`bignet.rs`**: The networking stack (using `reqwest`).

---

## License (BUPML)

This source code is released under the **BigC Universal Public Mandate License (BUPML)**.

*   **Freedom:** You are free to view, modify, fork, and use this code for personal or commercial projects.
*   **Sovereignty:** The names "BigC" and "BigRun" are reserved. If you fork this engine, you must rename your derivative work.
*   **Contribution:** Pull requests are welcome, especially for the `BigGuy` UI engine.

---

*"Flat is Better. Logic is Sovereign."*