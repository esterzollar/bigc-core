# Developer References & Codebase Map

This document serves as a comprehensive reference for the BigC engine codebase. It maps every source file, its functions, keywords involved, and its connections to other parts of the system.

---

## 1. `src/bighelp.rs`

**Purpose:**  
Provides built-in command-line assistance and static analysis tools for BigC developers. It handles the `whatis` and `show` CLI commands.

### Functions

*   **`whatis(keyword: &str)`**
    *   **Description:** Searches for a keyword definition in the `assets/textbook.json` file and prints a formatted manual page (Title, Description, Example).
    *   **Input:** A string slice representing the keyword to look up (e.g., "loop").
    *   **Output:** Prints to `stdout`.

*   **`show(filename: &str)`**
    *   **Description:** Performs a static analysis of a `.big` or `.guy` file. It tokenizes the source code and calculates statistics such as line count, logic complexity (if/or), function definitions (`doing`), view definitions (`view`), and math operations.
    *   **Input:** Path to the file to analyze.
    *   **Output:** Prints a statistical summary to `stdout`.

### Keywords & Tokens Handled

*   **CLI Keywords:** `whatis`, `show`.
*   **Analyzed Tokens:**
    *   `TokenType::Doing` (Function definitions)
    *   `TokenType::View` (UI Screen definitions)
    *   `TokenType::Draw` (GUI Rendering commands)
    *   `TokenType::If`, `TokenType::Or` (Control flow)
    *   `TokenType::Plus`, `TokenType::Minus`, `TokenType::Star` (Math intensity)

### Connections

*   **Internal Dependencies:**
    *   `crate::tokens::TokenType` (For identifying language constructs)
    *   `crate::lexer::Lexer` (For breaking down the file content)
*   **External Assets:**
    *   `assets/textbook.json`: The source of truth for the `whatis` command.

---

## 2. `src/bignet.rs`

**Purpose:**  
The networking core of BigC. It handles all HTTP/HTTPS requests (GET, POST), proxy configuration, user-agent spoofing, and HTML/JSON scraping. It acts as a wrapper around the `reqwest` and `scraper` crates.

### Functions

*   **`new() -> Self`**
    *   Initializes a new `BigNet` struct with empty configuration.

*   **`set_proxy(&mut self, url: &str)`**
    *   Configures the HTTP client to route traffic through a proxy server.
    *   **Triggers:** `build_client()` to apply changes.

*   **`set_user_agent(&mut self, agent: &str)`**
    *   Sets the `User-Agent` header string to mimic different browsers.
    *   **Triggers:** `build_client()` to apply changes.

*   **`add_header(&mut self, key: &str, value: &str)`**
    *   Adds a custom HTTP header (e.g., `Authorization`) to the client configuration.
    *   **Triggers:** `build_client()` to apply changes.

*   **`build_client(&mut self)`**
    *   Internal helper. Constructs the actual `reqwest::blocking::Client` using current settings (proxy, UA, headers, 30s timeout).

*   **`get(&mut self, url: &str) -> String`**
    *   Performs a blocking HTTP GET request.
    *   **Returns:** Response body text or an error string starting with "BigNet Error".

*   **`post(&mut self, url: &str, data: &str) -> String`**
    *   Performs a blocking HTTP POST request.
    *   **Content-Type:** Defaults to `application/x-www-form-urlencoded`.
    *   **Returns:** Response body text or an error string.

*   **`look_for(&self, pattern: &str, html: &str) -> String`**
    *   **Scraping Engine.** Uses CSS selectors to extract text or value attributes from an HTML string.
    *   **Returns:** The inner text or value of the *first* matching element.

*   **`look_at_json(&self, key: &str, json: &str) -> String`**
    *   **JSON Engine.** Parses a JSON string and retrieves the value associated with a top-level key.
    *   **Returns:** string value or empty string on failure.

### Keywords & Tokens Handled

*   `get web` (Calls `get`)
*   `get post` (Calls `post`)
*   `look for` (Calls `look_for`)
*   `look for json` (Calls `look_at_json`)
*   `proxy`, `user-agent`, `header` (Configuration setters)

### Connections

*   **External Crates:**
    *   `reqwest` (Blocking HTTP Client)
    *   `scraper` (HTML parsing)
    *   `serde_json` (JSON parsing)
*   **Usage:** Heavily used by `src/interpreter/get.rs` (for `get web` / `look for`) and `src/interpreter/actions.rs` (for network settings).

---

## 3. `src/lexer.rs`

**Purpose:**  
The Lexical Analyzer (Tokenizer) for BigC. It transforms raw source code (strings) into a stream of `Token` objects that the interpreter can understand. It handles white space, comments, string literals, numbers, and keyword identification.

### Functions

*   **`new(input: &str) -> Self`**
    *   Initializes the lexer with the source code input. Converts the string into a `Vec<char>` for efficient processing.

*   **`advance(&mut self)`**
    *   Moves the cursor to the next character in the input stream. Tracks `line` and `column` numbers for error reporting.

*   **`peek(&self) -> Option<char>`**
    *   Returns the character at the current position without advancing the cursor.

*   **`peek_next(&self) -> Option<char>`**
    *   Returns the character at the next position without advancing. Used for multi-character operators (e.g., `>=`, `=x`).

*   **`advance_char_raw(&mut self) -> Option<char>`**
    *   Similar to `advance` but returns the character and handles special tab-to-column conversion (tab = 4 columns). Used for reading blocks like `python3`.

*   **`tokenize(&mut self) -> Vec<Token>`**
    *   **Main Entry Point.** Iterates through the entire input and categorizes text into tokens.
    *   **Handles:**
        *   Comments (`#`) - Skips until newline.
        *   Operators (`=`, `&`, `+`, `-`, `*`, `/`, `^`, `>`, `<`, `{`, `}`, `(`, `)`, `[`, `]`, `.`, `:`, `$`, @).
        *   Special Operators (`=x` for NotEqual, `>=`, `<=`).
        *   String Literals (`"..."`) via `read_string()`.
        *   Numbers (`123.45`) via `read_number()`.
        *   Identifiers and Keywords via `read_identifier()`.

*   **`read_string(&mut self) -> Token`**
    *   Captures text between double quotes.
    *   **Supports Escaping:** `\n`, `\t`, `\r`, `\\`, `\"`.

*   **`read_number(&mut self) -> Token`**
    *   Captures sequences of digits and decimal points.
    *   **Supports Speed Suffix:** Recognizes numbers followed by `x` or `X` (e.g., `0.5x`) as `TokenType::SpeedVal`.

*   **`read_identifier(&mut self) -> Token`**
    *   Captures alphanumeric sequences (and underscores/dashes).
    *   **Keyword Mapping:** Matches the identifier against the internal keyword list.
    *   **Shorthand Handling:** Detects `s.loop`, `loop.s`, and `k.loop`.
    *   **Python Block Detection:** If `python3 start` is found, it calls `read_python_block()`.

*   **`read_python_block(&mut self) -> Token`**
    *   Captures raw text until `python3 end` is encountered. Returns a `TokenType::PythonCode` token.

### Keywords & Tokens Handled

*   **Every Keyword in BigC:** From `init` to `euler`.
*   **Symbols:** `=`, `&`, `+`, `-`, `*`, `/`, `^`, `>`, `<`, `{`, `}`, `(`, `)`, `[`, `]`, `.`, `:`, `$`, @).
*   **Literals:** `Identifier`, `Number`, `String`, `SpeedVal`, `PythonCode`.

### Connections

*   **Internal Dependencies:**
    *   `crate::tokens::{Token, TokenType}` (The data structure it produces)
*   **Usage:**
    *   `src/main.rs`: Calls `lexer.tokenize()` to start execution.
    *   `src/interpreter/mod.rs`: Used by the `heal_tokens` function and for parsing inline expressions.
    *   `src/bighelp.rs`: Used for file analysis.
    *   `src/interpreter/actions.rs`: Used for parsing `.bigenv` files via `attach`.

---

## 4. `src/luck.rs`

**Purpose:**  
The data generation engine ("The Luck Lab"). It provides methods to generate realistic random identities, locations, and system strings.

### Functions

*   **`new() -> Self`**
    *   Initializes the `BigLuck` struct with static datasets for names, zip codes, streets, and user agents.

*   **`get_first(&self) -> String`**
    *   Returns a random first name from the internal list of 100 common names.

*   **`get_last(&self) -> String`**
    *   Returns a random last name from the internal list of 100 common surnames.

*   **`get_zip(&self) -> String`**
    *   Returns a random 5-digit zip code (primarily Los Angeles area).

*   **`get_street(&self) -> String`**
    *   Returns a random street name from a list of major US thoroughfares.

*   **`get_user_agent(&self) -> String`**
    *   Returns a random, realistic browser identification string (User-Agent).

*   **`get_uuid(&self) -> String`**
    *   Generates a standard v4 UUID.

*   **`get_email(&self, first: &str, last: &str) -> String`**
    *   Generates a random email address using the provided names and a random number/provider suffix.

*   **`get_random_num(&self, min: i32, max: i32) -> String`**
    *   Returns a random integer within the specified range as a string.

### Keywords & Tokens Handled

*   `get luck name`, `get luck first`, `get luck last`, `get luck zip`, `get luck street`, `get luck ua`, `get luck random`, `get luck email`, `get luck uuid`.

### Connections

*   **External Crates:**
    *   `rand` (For randomness)
    *   `uuid` (For UUID generation)
*   **Usage:**
    *   `src/interpreter/get.rs`: The primary interface for `get luck` commands.
    *   `env_lib/picker.bigenv`: Uses `get luck random` to provide the `Random(min, max)` doing block.

---

## 5. `src/main.rs`

**Purpose:**  
The main entry point and CLI orchestrator for the BigC Language Engine (`bigrun`). It handles argument parsing, file reading, and directs execution to the appropriate module (Help, Logic, or GUI).

### Functions

*   **`main()`**
    *   **Argument Parsing:** Detects `whatis` and `show` commands for `BigHelp`.
    *   **Validation:** Ensures the input file has a supported extension (`.big`, `.guy`, or `.adkp`).
    *   **Lexing:** Instantiates the `Lexer` and generates the initial token stream.
    *   **Indentation Healing:** If the script contains `attach fixer`, it passes tokens through `interpreter.heal_tokens()` before execution.
    *   **Execution Modes:**
        *   **GUI Mode:** If the file ends in `.guy`, it enables the graphic engine and calls `interpreter.run_guy_direct()`.
        *   **Logic Mode:** Validates syntax and calls `interpreter.run(tokens)`.
    *   **Error Handling:** Checks `interpreter.last_error_pos` and exits with code 1 if a crash occurred.

### Keywords & Tokens Handled

*   **Internal Keywords:** `attach fixer` (Triggers token healing).
*   **File Extensions:** `.big`, `.guy`, `.adkp`.

### Connections

*   **Internal Modules:**
    *   `crate::lexer::Lexer`
    *   `crate::interpreter::Interpreter`
    *   `crate::bighelp::BigHelp`
    *   `crate::sound`
    *   `crate::guy_engine`
*   **Filesystem:** Reads source files from disk and ensures the `env_lib/` directory exists.

---

## 6. `src/sound.rs`

**Purpose:**  
The asynchronous audio engine for BigC. It manages sound playback, volume control, and system beeps using a background thread.

### Functions

*   **`start_sound_engine() -> Sender<SoundCommand>`**
    *   Initializes the `rodio` audio output stream and sink.
    *   Spawns a background thread that waits for commands via a channel.

*   **`SoundCommand` (Enum)**
    *   `Play(path)`, `SetVolume(vol)`, `Beep`.

### Keywords & Tokens Handled

*   `play sound`, `set volume`, `beep`.

### Connections

*   **External Crates:**
    *   `rodio`
*   **Usage:**
    *   `src/main.rs`: Initializes the engine if `.guy` mode is active.
    *   `src/interpreter/mod.rs`: Stores the `Sender` in the `Interpreter` struct.
    *   `src/interpreter/actions.rs`: Handles audio keywords.

---

## 7. `src/tokens.rs`

**Purpose:**  
Defines the vocabulary of the BigC language. It contains the `TokenType` enum and the `Token` struct.

### Functions

*   **`Token::new(token_type, line, column)`**
*   **`Token::is_number()`**
*   **`Token::is_operator()`**

### Keywords & Tokens Handled

Defines all `TokenType` variants (Keywords, Symbols, Literals).

### Connections

*   **Core Logic:** Foundation for Lexer and Interpreter.

---

## 8. `src/guy_engine/elements.rs`

**Purpose:**  
Defines rendering components for the BigGuy engine (Text, Buttons, Shapes, Images, Markdown).

### Functions

*   **`apply_style()`**: Merges named styles into local property maps.
*   **`draw_text()`**: Renders text with `cosmic-text` and texture caching.
*   **`draw_button()`**: Renders interactive buttons with hover/click detection.
*   **`draw_rect()`**: Renders rectangles/rounded rectangles.
*   **`draw_circle()`**, **`draw_triangle()`**, **`draw_line()`**: Shape rendering.
*   **`draw_input()`**: 2-way data binding for text input.
*   **`draw_image()`**: Raster rendering with rotation, scale, tint, and tags.
*   **`draw_scroll_area()`**: Scrollable container wrapper.
*   **`draw_markdown()`**: Rich text rendering with sub-style support (e.g., `Style.h1`).

### Keywords & Tokens Handled

*   `draw` + [text, button, input, rectangle, rounded, circle, triangle, line, image, scroll, markdown].
*   Properties: `size`, `font`, `fill`, `stroke`, `border`, `radius`, `alpha`, `layer`, `tag`, `rotate`, `scale`, `tint`, `width`, `height`, `from`, `to`, `at`.

### Connections

*   `crate::guy_engine::window`: Uses `BigGuyApp`.
*   `crate::interpreter::Interpreter`: Pulls variables and styles.

---

## 9. `src/guy_engine/mod.rs`

**Purpose:**  
The module orchestrator for the BigGuy graphic engine. it handles direct launch from `.guy` files and the `start guy` command.

### Functions

*   **`run_direct(interpreter, filename, tokens)`**: Entry point for `.guy` files. Registers views and launches `eframe`.
*   **`handle_start(interpreter, i, tokens)`**: Processes `start guy [ViewName]`. Manages window initialization and font registration.

### Keywords & Tokens Handled

*   `start guy`.

### Connections

*   `crate::guy_engine::window::BigGuyApp`, `crate::guy_engine::elements`, `crate::interpreter::Interpreter`.

---

## 10. `src/guy_engine/window.rs`

**Purpose:**  
The core application loop and window management system for the BigGuy engine. It implements the `eframe::App` trait and handles the conversion of BigC `view` tokens into an interactive graphical interface.

### Functions

*   **`BigGuyApp::update()`**: 60 FPS loop. Syncs Window size, mouse pos, keyboard state, and global clicks. Manages "State Swapping" for interaction tags.
*   **`run_init()`**: Executes view `init` blocks.
*   **`get_cached_image_file()`**, **`get_cached_texture()`**: Texture management and eviction.
*   **`render_recursive()`**: Main token-to-UI dispatcher.
*   **`handle_draw()`**: Dispatches to `elements.rs`.
*   **`css_color()`**: Hex-to-RGB conversion.

### Keywords & Tokens Handled

*   `view`, `init`, `if`, `or`, `set window`, `draw row`, `draw column`, `$MouseX`, `$MouseY`, `$WindowWidth`, `$WindowHeight`, `$Delta`.

### Connections

*   `crate::guy_engine::elements`, `crate::interpreter::Interpreter`.

---

## 11. `src/interpreter/actions.rs`

**Purpose:**  
The primary action dispatcher for the BigC interpreter. It handles core language verbs, I/O operations, network settings, and unified variable assignment.

### Functions

*   **`handle_action(i, tokens)`**: The "Giant Switch" for imperative commands. Handles `print`, `wait`, `take`, `ask input`, `reset`, `attach`, `user-agent`, `proxy`, `header`, `split`, `event`, `build task`, `asset load`, `replace`, `play sound`, and `beep`.
*   **`handle_assignment(i, tokens, var_name)`**: Manages the `=` operator. Detects Constructors (Warp, Solve), In-line Math, and Literal Assignment.

### Keywords & Tokens Handled

*   `print`, `wait`, `take`, `ask input`, `reset`, `attach`, `user-agent`, `proxy`, `header`, `split`, `event`, `build task`, `asset load`, `replace`, `play sound`, `beep`.
*   **Symbols:** `=`, `&`, `at (@)`, `by`, `with`.

### Connections

*   **Pointer Logic:** Explicitly uses `*i -= 1` at the end of Handlers to adjust for the main loop's automatic increment, ensuring the next command is not skipped.
*   `crate::lexer::Lexer`, `crate::interpreter::mod`, `crate::sound`, `crate::bignet`.

---

## 12. `src/interpreter/architect.rs`

**Purpose:**  
Provides system inspection and validation tools ("The Architect"). Currently focused on filesystem existence checks.

### Functions

*   **`handle_architect(i, tokens)`**
    *   **Role:** Processes the `check if` command.
    *   **Logic:** Checks if a target path exists using `std::path::Path::exists()`.
    *   **Returns:** "true" or "false".

### Keywords & Tokens Handled

*   `check if "[Path]" here`
*   **Result Pipeline:** `& set as {Var}`.

### Connections

*   **Internal Dependencies:** `crate::interpreter::mod`.
*   **Standard Library:** `std::path::Path`.

---

## 13. `src/interpreter/biew.rs`

**Purpose:**  
The experimental transpiler for Web UI components. It converts BigC component syntax (`.biew`) and stylist sheets (`.bss`) into standard HTML5 and CSS3.

### Functions

*   **`transpile_biew(content: &str) -> String`**
    *   **Role:** Entry point for `.biew` files.
    *   **Workflow:**
        1. Tokenizes the input.
        2. Iterates line-by-line using `process_line`.
        3. Maintains a stack of HTML tags to ensure correct closing.
        4. Injected a default HTML5 boilerplate.
    *   **Returns:** A complete HTML document string.

*   **`transpile_bss(content: &str) -> String`**
    *   **Role:** Entry point for stylist sheets.
    *   **Returns:** A raw CSS string.

*   **`process_line(tokens, raw_line, state)`**
    *   **Role:** The core line translator. Maps keywords like `box`, `header`, `text` to HTML tags.

*   **`transpile_bss_line(tokens) -> String`**
    *   Translates style properties into CSS. Handles `center` (Flexbox) and automatic `px` units.

### Keywords & Tokens Handled

*   **Biew:** `box`, `header`, `subheader`, `title`, `text`, `button`, `link`, `image`, `item`, `raw`, `end`.
*   **BSS:** `style`, `center`, `on hover`, `end style`.

### Connections

*   **Internal Dependencies:** `crate::lexer::Lexer`.
*   **Usage:** `bigweb.rs` (reply file), `get.rs` (look for).

---

## 14. `src/interpreter/bigweb.rs`

**Purpose:**  
The BigWeb server engine. It transforms BigC scripts into functional web servers using `tiny_http`. It manages routing, request handling, rate limiting, and response generation.

### Functions

*   **`handle_use_sbig(i, tokens)`**
    *   **Role:** Unlocks the web server module.
    *   **Trigger:** `use web` or `use sbig`.

*   **`handle_server_config(i, tokens)`**
    *   **Role:** Processes server tuning commands.
    *   **Settings:**
        *   `control workers [N]`: Sets thread count (reserved).
        *   `control limit [N] per mins`: Configures rate limiting.
        *   `control record @"file.log"`: Enables request logging to disk.
        *   `control ssl @"cert" @"key"`: Sets up SSL config.

*   **`handle_on(i, tokens)`**
    *   **Role:** Defines a URL route.
    *   **Syntax:** `on [get/post] "/path" run [Doing]`.
    *   **Logic:** Registers the mapping in the interpreter's shared `routes` map.

*   **`handle_start_server(i, tokens)`**
    *   **Role:** Launches the blocking HTTP server.
    *   **Logic:**
        1. Binds to `127.0.0.1:[Port]`.
        2. Enters a loop waiting for `incoming_requests`.
        3. **Rate Limiting:** Checks against `rate_limit` settings.
        4. **Route Matching:** Matches URL and Method (supports `+` wildcard).
        5. **State Injection:** populates `$RequestBody`, `$RequestPath`, `$RequestMethod`, and `$RequestExtra`.
        6. **Execution:** Runs the mapped `doing` block.
        7. **Transpilation:** If replying with `.biew` or `.bss`, it calls the `Biew` transpiler.
        8. **Response:** Sends the final body, status code, and headers back to the client.

*   **`handle_reply(i, tokens)`**
    *   **Role:** Configures the response for the current request.
    *   **Syntax:**
        *   `reply with "[Text]"`: Direct content.
        *   `reply file "[Path]"`: Serves a file (transpiles if Biew/BSS).
        *   `reply point [Code]`: Sets status code (e.g., 404).
        *   `reply note "[Key]" as "[Value]"`: Sets HTTP headers.

### Keywords & Tokens Handled

*   `use web`, `use sbig`, `control`, `on get`, `on post`, `start server`, `reply`, `with`, `file`, `point`, `note`, `workers`, `limit`, `record`, `ssl`.
*   **System Variables:** `RequestBody`, `RequestPath`, `RequestMethod`, `RequestExtra`, `Sbig_Response_Body`, `Sbig_Response_File`.

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::biew`: For on-the-fly HTML/CSS generation.
    *   `crate::interpreter::mod`: Uses `run` to execute logic blocks and `set_variable` for request data.
*   **External Crates:**
    *   `tiny_http`: The underlying HTTP server implementation.
    *   `chrono`: For logging timestamps.

---

## 15. `src/interpreter/bit.rs`

**Purpose:**  
The data security and cryptography module ("The Bit Lab"). It provides tools for both native "Textbook" encryption (NEBC) and industry-standard AES encryption.

### Functions

*   **`handle_bit(i, tokens)`**
    *   **Role:** Main entry point for the `bit` keyword.
    *   **Actions:** Dispatches to `handle_bit_code` (NEBC encode), `handle_bit_decode` (NEBC decode), `handle_bit_aes` (AES), or `handle_bit_demon` (Scrambling).

*   **`handle_bit_code(i, tokens)`** / **`handle_bit_decode(i, tokens)`**
    *   **Role:** Processes native BigC encryption.
    *   **Syntax:** `bit [code/decode] "[Text]" with "[Key]" nebc`.
    *   **Logic:** Uses the `holmes_math_cipher`.

*   **`handle_bit_demon(i, tokens)`**
    *   **Role:** Irreversibly scrambles text.
    *   **Logic:** Injects two random characters between every character of the source text.

*   **`handle_bit_aes(i, tokens)`**
    *   **Role:** Industry-standard AES-256-CBC encryption/decryption.
    *   **Syntax:** `bit aes [encrypt/decrypt] "[Data]" key "[32ByteKey]" iv "[16ByteIV]"`.
    *   **Logic:** Uses the `aes` and `cbc` crates with PKCS7 padding. Encoded results are Base64 strings.

*   **`holmes_math_cipher(text, key, encrypt) -> String`**
    *   **Role:** The NEBC algorithm.
    *   **Logic:** A rolling positional shift cipher. It shifts each character's position in an alphanumeric alphabet by a value derived from the character index and the corresponding byte in the key.

*   **`aes_encrypt(data, key, iv) -> String`** / **`aes_decrypt(data, key, iv) -> String`**
    *   Internal helpers for performing AES operations and Base64 conversion.

### Keywords & Tokens Handled

*   `bit`, `code`, `decode`, `aes`, `encrypt`, `decrypt`, `nebc`, `demon`, `key`, `iv`, `with`.

### Connections

*   **External Crates:**
    *   `aes` (AES block cipher).
    *   `cbc` (Cipher Block Chaining mode).
    *   `block-padding` (PKCS7).
    *   `base64` (For string-safe representation of binary ciphertexts).
    *   `rand` (For random noise in `demon`).
*   **Usage:** Used by developers for securing passwords, sensitive database fields, or obfuscating UI elements.

---

## 16. `src/interpreter/bmath.rs`

**Purpose:**  
The pattern matching and regex validation module ("Bmath"). It allows developers to check strings against regular expressions using the Rust `regex` engine.

### Functions

*   **`handle_bmath(i, tokens)`**
    *   **Role:** Processes the `bmath` keyword.
    *   **Syntax:** `bmath "[Text]" @"[Pattern]"`.
    *   **Logic:**
        1. Extracts the source text.
        2. Validates `@` spacing strictness.
        3. Extracts the regex pattern.
        4. Compiles and executes the regex.
        5. Returns "true" or "false" via the `handle_set_as_multiple` result pipeline.

### Keywords & Tokens Handled

*   `bmath`, `at (@)`.

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `get_token_value`, `validate_at_strictness`, and `handle_set_as_multiple`.
*   **External Crates:**
    *   `regex` (The underlying matching engine).

---

## 17. `src/interpreter/books.rs`

**Purpose:**  
The core filesystem interaction module ("The Books Lab"). It handles all direct file I/O operations including reading, writing, appending, and file management (copy, move, delete).

### Functions

*   **`handle_books(i, tokens)`**
    *   **Role:** Main entry point for file-related keywords.
    *   **Actions:**
        *   `open`: Reads an entire file into a variable. Sets `$BugType` if the file is missing.
        *   `write`: Overwrites (or creates) a file with specified content.
        *   `add`: Appends content to an existing file (or creates it).
        *   `delete file`: Removes a file or directory.
        *   `copy file`: Duplicates a file to a new destination.
        *   `move file`: Renames or moves a file.

### Keywords & Tokens Handled

*   `open`, `write`, `add`, `delete`, `copy`, `move`, `file`, `to`, `at (@)`.
*   **Error Reporting:** Populates `$BugType` with "File Error", "Open Error", "Write Error", etc.

### Connections

*   **Pointer Logic:** Correctly implements `*i -= 1` after complex consumption to ensure the next command is processed by the main loop.
*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `get_complex_value`, `interpolate_string`, `validate_at_strictness`, and `handle_set_as_multiple`.
*   **Standard Library:**
    *   `std::fs`: For low-level file operations.
    *   `std::io::Write`: For buffered writing.

---

## 18. `src/interpreter/control.rs`

**Purpose:**  
The central orchestration module for program execution flow. It handles logical branching (If/Or), looping (Standard, Smart, For-Each), function and view registration, and animation stepping.

### Functions

*   **`handle_control(i, tokens)`**
    *   **Role:** Main dispatcher for control-flow keywords.
    *   **Logic Branching:**
        *   `if`: Evaluates conditions. If false, it intelligently skips the rest of the line and any indented blocks or lines starting with `&`.
        *   `or`: Executes only if the previous `if` condition was unmet.
    *   **Registration:**
        *   `start doing`: Registers function tokens and parameters in shared memory.
        *   `start view`: Registers BigGuy UI layout tokens.
        *   `start style`: Registers reusable UI property maps.
    *   **Looping Engine:**
        *   `start loop`: Initializes a `LoopInfo` object and pushes it to the loop stack. Supports indentation validation.
        *   `.r.N`: Handles smart safety limits (e.g., `start loop.r.50`).
        *   `loop on`: Implements **For-Each** logic for Backpacks (Lists) and Maps. Automatically binds items/keys/values to variables.
        *   `keep`: Evaluates the loop continuation condition. Handles **Refuel** logic (`keep loop`).
        *   `stop loop` / `loop.s`: Breaks out of the current loop stack.
    *   **Program Exit:**
        *   `stop run`: Terminates the engine process.
        *   `addrun end`: Alias for process termination.

*   **`handle_step(i, tokens)`**
    *   **Role:** Implements frame-independent animation.
    *   **Syntax:** `step {Var} towards {Target} speed {Nx}`.
    *   **Logic:** Uses exponential damping (decay) to move a value. Calculation: `new = current + (diff * (1.0 - exp(-speed * 5.0 * dt)))`. Snap-to-target occurs if the difference is < 0.5.

### Keywords & Tokens Handled

*   `if`, `or`, `start`, `doing`, `view`, `style`, `loop`, `s.loop`, `loop.s`, `on`, `as`, `keep`, `stop`, `run`, `end`, `step`, `towards`, `speed`.
*   **Internal Shorthands:** `s.loop` (start), `loop.s` (stop), `k.loop` (keep).

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `evaluate_condition`, `get_token_value`, `get_variable`, `set_variable`, and shared state access (functions, views, styles).
    *   `crate::guy_engine`: `handle_step` uses `current_dt` from the GUI loop for stable animation.

---

## 19. `src/interpreter/dbig.rs`

**Purpose:**  
The module for Native BigC Database (DBIG) interaction. It implements atomic Key-Value storage with spin-lock safety, allowing multiple processes to share a database file without corruption.

### Functions

*   **`handle_dbig(i, tokens)`**
    *   **Role:** Main entry point for the `dbig` keyword.
    *   **Actions:** Dispatches to specific handlers for `get`, `set`, `remove`, and `check`.

*   **`handle_dbig_get(i, tokens)`**
    *   **Role:** Retrieves values associated with a key.
    *   **Logic:** Reads the file, parses the block format, and returns all values as a list.

*   **`handle_dbig_set(i, tokens)`**
    *   **Role:** Creates or updates a database key.
    *   **Atomic Safety:** Implements **Spin-Locking** by creating a `.lock` file. Retries up to 100 times before giving up.
    *   **Logic:** Replaces the existing block for the key with new values.

*   **`handle_dbig_remove(i, tokens)`**
    *   **Role:** Deletes a key and all its associated values from the database.
    *   **Atomic Safety:** Uses the same spin-lock mechanism as `set`.

*   **`handle_dbig_check(i, tokens)`**
    *   **Role:** Verifies data without retrieving it.
    *   **Modes:**
        1. **Existence:** `dbig check "Key"`.
        2. **List Search:** `dbig check "Item" of "Key"`.
        3. **Scanner:** `dbig check keys value < 10` (Calls `handle_dbig_scan`).

*   **`handle_dbig_scan(i, tokens)`**
    *   **Role:** Searches the entire database for keys matching a numeric or string condition.
    *   **Syntax:** `dbig check keys value [Operator] [Target]`.
    *   **Logic:** Iterates through all blocks and evaluates the condition against stored values.

*   **Parsing Helpers:**
    *   `parse_dbig_content`: Extracts block data.
    *   `update_dbig_content`: Generates new file content with updated blocks.
    *   `remove_dbig_block`: Filters out a specific key block.
    *   `scan_dbig_values`: Logic for the multi-key scanner.

### Keywords & Tokens Handled

*   `dbig`, `get`, `set`, `remove`, `check`, `keys`, `value`, `as`, `of`, `at (@)`.
*   **Symbols:** `>`, `<`, `=`. 

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `get_token_value`, `get_complex_value`, `validate_at_strictness`, and `handle_set_as_multiple`.
*   **Filesystem:** Directly manages `.dbig` and `.dbig.lock` files.

---

## 20. `src/interpreter/dbr.rs`

**Purpose:**  
The module for Universal Database (DBR) interaction, providing BigC with standard SQLite capabilities. It acts as a wrapper around the `rusqlite` crate.

### Functions

*   **`handle_use_sql(i, tokens)`**
    *   **Role:** Unlocks the SQL module.
    *   **Trigger:** `use sql`.

*   **`handle_run_sql(i, tokens)`**
    *   **Role:** Executes a non-query SQL command (INSERT, UPDATE, DELETE, CREATE).
    *   **Syntax:** `run sql "[Query]" on "[DBPath]"`.
    *   **Logic:** Opens a connection to the SQLite file and executes the raw query string.

*   **`handle_get_sql(i, tokens)`**
    *   **Role:** Executes a SELECT query and retrieves data.
    *   **Syntax:** `get sql "[Query]" on "[DBPath]"`.
    *   **Logic:** 
        1. Prepares the SQL statement.
        2. Iterates through the results.
        3. Flattens all returned columns and rows into a single `Vec<String>`.
        4. Returns the result via the `handle_set_as_multiple` pipeline.

### Keywords & Tokens Handled

*   `use sql`, `run sql`, `get sql`, `on`.

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `get_token_value`, `interpolate_string`, and `handle_set_as_multiple`.
*   **External Crates:**
    *   `rusqlite`: The SQLite driver.

---

## 21. `src/interpreter/get.rs`

**Purpose:**  
The "Universal Getter" and data transformer module. It processes the `get` and `look` keywords, providing functionality for data validation, networking, random generation, and string modification.

### Functions

*   **`handle_get(i, tokens)`**
    *   **Role:** Main entry point for `get`, `look`, and `check`.
    *   **Validation Modes (`as` keyword):**
        *   `email`, `number`, `url`, `alphanumeric`: Boolean checks.
        *   `clean`: Trims whitespace.
        *   `bigcap`, `lower`: Case conversion.
        *   `floor`, `ceil`, `round`, `abs`, `positive`: Math rounding/signs.
        *   `smaller`, `bigger`, `between`: Comparisons and clamping.
        *   `len`, `length`, `count`: Calculates size of strings or lists.
    *   **Retrieval Modes:**
        *   `get web`: Blocking HTTP GET via BigNet.
        *   `get post`: Blocking HTTP POST via BigNet.
        *   `get time`: Returns `unix` timestamp, high-res `tick`, or frame `delta`.
        *   `get luck`: Identity generation (name, email, random range, etc.).
        *   `get markdown`: Converts Markdown text into HTML.
        *   `get setting`: Reads system environment variables.
        *   `get {Map}`: JSON packing of a BigC Map.
        *   `get from {List} @Index`: Positional list access (1-based).
        *   `get count of {List}`: Legacy list sizing.
    *   **In-line Math:** Processes MDAS expressions with operator precedence.

*   **`look for` (CSS Scraping / JSON Extraction):**
    *   **Syntax:** `look for [Selector] @[Source]`.
    *   **Selector `all`**: returns the entire source.
    *   **JSON Mode:** Single-level key lookup.
    *   **CSS Mode:** standard selector extraction via BigNet.

### Keywords & Tokens Handled

*   `get`, `look`, `for`, `in`, `json`, `all`, `as`, `from`, `of`, `at (@)`, `with`, `replace`.
*   **Constants:** `pi`, `euler`.

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `get_complex_value`, `get_token_value`, `interpolate_string`, and `handle_set_as_multiple`.
    *   `crate::bignet`: Integration for scraping and HTTP.
    *   `crate::luck`: Integration for random generation.
*   **External Crates:**
    *   `pulldown-cmark`: For Markdown transpilation.
    *   `regex`: For string validation.

---

## 22. `src/interpreter/helpers.rs`

**Purpose:**  
The structural utility module for the Interpreter. it provides essential logic for string interpolation, conditional evaluation, token length calculation, and spacing validation.

### Functions

*   **`get_token_len(token) -> usize`**
    *   **Role:** Calculates the visual length of a token's value. Used for strict spacing validation.
    *   **Logic:** Accounts for quotes around strings and formatting of numbers.

*   **`validate_at_strictness(i, tokens) -> bool`**
    *   **Role:** Enforces the Mandatory Spacing Rule for the `@` pointer.
    *   **Check 1:** Ensures at least one space exists *before* `@`.
    *   **Check 2:** Ensures zero spaces exist *after* `@` (the target must be attached).

*   **`handle_set_as_multiple(i, tokens, results)`**
    *   **Role:** Implements the `& set as` connector for multi-return commands.
    *   **Modes:**
        *   **List Mode:** Bundles all results into a JSON array string if the `list` keyword is present.
        *   **Standard Mode:** Binds individual results to a sequence of braced variables (e.g., `{Var1} {Var2}`).

*   **`evaluate_condition(tokens) -> bool`**
    *   **Role:** Resolves logical comparisons (`if` conditions).
    *   **Logic:**
        1. **Special UI States:** Handles `btnclick`, `click`, `hover "Tag"`, `press "Tag"`, `drag "Tag"`, and `keydown "Key"`.
        2. **Binary Comparisons:** Executes `>` , `<`, `=`, `>=` , `<=`, and `=x` (NotEqual) for both numbers and strings.
        3. **Truthiness:** If no operator is present, calls `evaluate_expression` (True if > 0).

*   **`evaluate_expression(tokens) -> f64`**
    *   **Role:** Performs flat, left-to-right math evaluation for basic truthiness checks.

*   **`get_complex_value(i, tokens) -> String`**
    *   **Role:** Retrieves a value while resolving interpolators (`$`), length operators (`len`), and system variables (`$Delta`, `$Tick`).

*   **`interpolate_string(text) -> String`**
    *   **Role:** The core interpolation engine. Replaces `$VarName` or `${VarName}` with their current values within a string.

*   **`validate_syntax(tokens) -> bool`**
    *   **Role:** Pre-execution check. Ensures the reserved keyword `Val` is not used as a variable name.

*   **`extract_braced_name(i, tokens) -> String`**
    *   **Role:** Utility to pull variable names from `{Container}` syntax.

### Keywords & Tokens Handled

*   `set as`, `list`, `len`, `at (@)`.
*   **Conditionals:** `btnclick`, `click`, `hover`, `press`, `drag`, `keydown`.
*   **System Vars:** `$Delta`, `$Tick`, `$MouseX`, `$MouseY`.

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Core struct implementation.
    *   `crate::tokens`: For token pattern matching.
*   **System:** Directly interfaces with `std::io` for input/output synchronization.

---

## 23. `src/interpreter/lists.rs`

**Purpose:**  
The collection management module for Backpacks (Lists). It provides tools for mutating JSON-based arrays and retrieving items by their 1-based index.

### Functions

*   **`handle_list(i, tokens)`**
    *   **Role:** Processes the `list` keyword for mutations.
    *   **Actions:**
        *   `add`: Appends a string value to the end of the JSON array.
        *   `remove`: Filters the list to remove all items exactly matching the value.
        *   `cut`: Deletes an item at a specific 1-based index.
        *   `sort`: Sorts the array alphabetically.
        *   `insert`: Places a value at a specific 1-based index, shifting subsequent items.

*   **`handle_get_from(i, tokens)`**
    *   **Role:** Retrieves a single item from a list.
    *   **Syntax:** `get from {List} at [Index]`.
    *   **Logic:**
        1. Parses index as float first (for smart-parse of "1.0").
        2. Converts to 0-based internal index.
        3. Returns the value or the literal string "nothing" if out of bounds.

*   **`handle_get_count(i, tokens)`**
    *   **Role:** Returns the number of items in a list.
    *   **Syntax:** `get count of {List}`.

### Keywords & Tokens Handled

*   `list`, `add`, `remove`, `cut`, `sort`, `insert`, `get from`, `count of`, `at`, `on`, `by`, `to`, `from`, `at (@)`.

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `get_token_value`, `get_variable`, `set_variable`, and `handle_set_as_multiple`.
*   **External Crates:**
    *   `serde_json`: For parsing and serializing list strings.

---

## 24. `src/interpreter/maps.rs`

**Purpose:**  
The data management module for Maps (Dictionaries). It provides functionality for creating, updating, and querying JSON-based key-value stores.

### Functions

*   **`handle_map(i, tokens)`**
    *   **Role:** Main dispatcher for the `map` keyword.
    *   **Actions:** Dispatches to specific handlers for `set`, `get`, `check`, `remove`, and `merge`.

*   **`handle_map_set(i, tokens)`**
    *   **Role:** Adds or updates a key in a Map.
    *   **Syntax:** `map set "[Key]" as "[Value]" @{MapVar}`.
    *   **Logic:** Resolves the Map variable, parses it as a JSON object, inserts the key-value pair, and saves the updated string.

*   **`handle_map_get(i, tokens)`**
    *   **Role:** Retrieves the value for a specific key.
    *   **Syntax:** `map get "[Key]" of {MapVar}`.
    *   **Returns:** The string value or the literal string `"nothing"` if the key is missing.

*   **`handle_map_check(i, tokens)`**
    *   **Role:** Checks for the existence of a key.
    *   **Syntax:** `map check "[Key]" of {MapVar}`.
    *   **Returns:** `"true"` or `"false"` via the result pipeline.

*   **`handle_map_remove(i, tokens)`**
    *   **Role:** Deletes a key-value pair from a Map.
    *   **Syntax:** `map remove "[Key]" from @{MapVar}`.

*   **`handle_map_merge(i, tokens)`**
    *   **Role:** Combines two maps.
    *   **Syntax:** `map merge {SourceMap} @{TargetMap}`.
    *   **Logic:** Overwrites existing keys in the target with values from the source.

### Keywords & Tokens Handled

*   `map`, `set`, `get`, `check`, `remove`, `merge`, `as`, `of`, `from`, `at (@)`.

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `get_token_value`, `get_variable`, `set_variable`, `extract_braced_name`, and `handle_set_as_multiple`.
*   **External Crates:**
    *   `serde_json`: For parsing and modifying map strings.

---

## 25. `src/interpreter/math_elements.rs`

**Purpose:**  
The advanced math engine for BigC. It implements the high-precision recursive solver (`S[]`) and provides handles for trigonometric, logarithmic, and statistical functions.

### Functions

*   **`handle_solve_constructor(i, tokens) -> String`**
    *   **Role:** Entry point for the `S[]` Solver.
    *   **Logic:** Captures all tokens between `[` and `]` (handling nested brackets) and passes them to `evaluate_math_recursive`.
    *   **Returns:** Formatted result string (strips `.0` from whole numbers).

*   **`evaluate_math_recursive(tokens) -> f64`**
    *   **Role:** The core PEMDAS engine.
    *   **Process:**
        1. **Pass 1 (Atomization):** Resolves Parentheses `( )` recursively. Resolves Functions (`sqrt`, `sin`, `cos`, `tan`, `abs`, `log`, `minimum`, `maximum`, `len`).
        2. **Pass 2 (Exponents/Modulo):** Processes `^` and `remainder` (mod) left-to-right.
        3. **Pass 3 (Multiplication/Division):** Processes `*` and `/`. Handles "DivisionByZero" error state.
        4. **Pass 4 (Addition/Subtraction):** Processes `+` and `-`.
    *   **Returns:** Final floating-point result.

*   **`resolve_val(token_type) -> f64`**
    *   Internal helper. Resolves numbers, speed values (Nx), and variable names into doubles.

### Keywords & Tokens Handled

*   `S[...]` (Solve constructor).
*   **Operators:** `+`, `-`, `*`, `/`, `^`, `remainder`.
*   **Functions:** `sqrt`, `sin`, `cos`, `tan`, `abs`, `log`, `minimum`, `maximum`, `len`.
*   **Constants:** `pi`, `euler`.

### Connections

*   **Internal Dependencies:**
    *   `crate::interpreter::mod`: Uses `get_variable` and `get_complex_value`.
    *   `crate::interpreter::actions`: `handle_assignment` calls the solver if `S[]` or math operators are detected.
*   **Usage:** Powers the GPU-accelerated physics in `handle_step` and all coordinate calculations in `src/guy_engine`.

---

## 26. `src/interpreter/mod.rs`

**Purpose:**  
The core architectural hub of the BigC engine. It defines the `Interpreter` struct, manages shared and local execution states, handles multi-threaded logic, and coordinates all sub-modules (actions, control, math, etc.).

### Functions

*   **`new() -> Self`**
    *   **Role:** Constructor for the Interpreter.
    *   **State Initialization:** Sets up shared thread-safe pointers (`Arc<RwLock>`) for variables, functions, views, styles, assets, and routes. Initializes the event queue and UI interaction maps.

*   **`run(tokens: Vec<Token>)`**
    *   **Role:** The primary execution loop.
    *   **Logic:**
        1. Tracks recursion depth (limit: 100).
        2. Pushes a new local scope if inside a function call.
        3. Iterates through tokens and dispatches to sub-module handlers (`handle_action`, `handle_books`, `handle_control`, etc.).
        4. **Error HALT:** Automatically stops and reports errors if `last_bug_found` is true (unless an explicit error check follows).
        5. Handles `return` logic and variable shadowing.

*   **`heal_tokens(tokens) -> Vec<Token>`**
    *   **Role:** Implements "The Fixer" logic for indentation forgiveness.
    *   **Logic:** Scans tokens and automatically injects `keep 1` tokens to close loops based on column indentation changes.

*   **`get_variable(name) -> Option<String>`**
    *   **Role:** Resolves variable values with scope priority.
    *   **Priority:** 1. System Variables (`Tick`, `Delta`, `MouseX`, `MouseY`, `DragX`, `DragY`). 2. Dot Notation (`Object.Prop`). 3. Local Scopes (Reverse stack search). 4. Global Shared Variables.

*   **`set_variable(name, value)`**
    *   **Role:** Writes data to memory.
    *   **Scoping:** Variables ending in global suffixes (`Raw`, `Content`, `Layout`, `Html`, `Biew`) or special system names are always written to the shared global map. Other variables are written to the current local scope if inside a `doing` block.

*   **`consume_math(i, tokens) -> f32`**
    *   **Role:** Specialized math parser for UI coordinates.
    *   **Logic:** Collects tokens until a UI keyword or connector is hit, resolves interpolations, and calculates the result.

*   **`report_error(message, line, col)`**
    *   **Role:** Formats and prints engine errors with a visual "BigC ERROR" box, including the source line and call stack.

*   **`handle_dot_assignment(i, tokens, obj_name, prop_name)`**
    *   **Role:** Internal handler for `Object.Prop = Value`. Modifies the underlying JSON string of the object.

### Struct Fields (Architecture)

*   **Shared State (`Arc<RwLock<...>>`):** `variables`, `functions`, `blueprints`, `views`, `styles`, `assets`, `routes`, `event_queue`.
*   **Interaction Maps:** `clicked_tags`, `hovered_tags`, `pressed_tags`, `dragged_tags` (and their `last_` frame counterparts).
*   **Local State:** `loop_stack`, `local_scopes`, `net` (BigNet), `luck` (BigLuck).
*   **Flags:** `sql_enabled`, `sbig_enabled`, `pybig_enabled`, `guy_enabled`, `autolayering_enabled`.

### Connections

*   **Root Dependencies:** `crate::tokens`, `crate::lexer`.
*   **Sub-Modules:** Orchestrates all files in `src/interpreter/`.
*   **External Integration:**
    *   `src/main.rs`: Entry point for script execution.
    *   `src/guy_engine`: Communicates UI state and provides frame-delta for animation.
    *   `src/sound`: Sends commands to the background audio thread.

---

## 27. `src/tokens.py`

**Purpose:**  
A Python implementation of the BigC tokenization logic. It defines the `TokenType` and `Token` classes, primarily used as a bridge for the `pyBig` system or as a reference for external scripts interacting with BigC source files.

### Classes & Functions

*   **`TokenType` (Enum)**
    *   Defines core BigC keywords and symbols using Python's `Enum` and `auto()`.
    *   **Keywords included:** `DEFINATION`, `ITEM`, `GET`, `SET`, `PRINT`, `IF`, `START`, `KEEP`, `END`, etc.
    *   **Literals:** `NUMBER`, `STRING`, `IDENTIFIER`.
    *   **Symbols:** `ASSIGN (=)`, `AMPERSAND (&)`, `PLUS (+)`, etc.

*   **`Token` (Class)**
    *   **`__init__(type, value, line)`**: Constructor for a Python Token instance.
    *   **`__repr__()`**: Returns a string representation of the token for debugging (e.g., `Token(STRING, 'Hello')`).

### Connections

*   **System Integration:** Mirrors the definitions in `src/tokens.rs`.
*   **Usage:** Used within the `pyBig` subsystem to allow Python scripts to understand and manipulate BigC code structures.
