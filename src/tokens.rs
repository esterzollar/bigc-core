#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Keywords
    Init,
    Print,
    Update,
    Get,
    Set,
    Global,
    As,
    If,
    Or,
    Take,
    Wait,
    Type,
    Export,
    Use,
    Lab,
    Attach,
    Stop,
    Warp,
    Run,
    Start,
    Build,
    Pack,
    Unpack,
    Command,
    // BigNet
    Bignet,
    Netloop,
    Look,
    For,
    In,
    With,
    Proxy,
    UserAgent,
    Header,
    Post,
    Split,
    By,
    Json,
    Luck,
    Random,
    Email,
    UUID,
    Loop,
    Sloop,
    Loops,
    Doing,
    Return,
    Keep,
    Blueprint,
    Pin,
    New,
    Addrun,
    Andrun,
    End,
    Any,
    Bug,
    Found,
    Reset,
    Len,

    // Time & Events
    BigTick,
    BigDelta,
    Event,
    Push,
    Pop,

    // File I/O
    Open,
    Book,
    Write,
    To,
    Add,
    Delete,
    Copy,
    Create,
    Folder,
    File,

    // Lists & Maps Power
    List,
    From,
    Of,
    Sort,
    Insert,
    Merge,

    // Architect
    Check,
    Here,
    Cut,
    AtWord, // The 'at' keyword

    // DBR (SQLite)
    Sql,
    On,

    // DBB (BigC Native DB)
    Dbig,
    Remove,
    Keys,
    Value,

    // BigGuy (Native Interface)
    Guy,
    View,
    Refresh,
    Draw,
    Rectangle,
    Rounded,
    Circle,
    Triangle,
    Line,
    Path,
    Button,
    Move,
    Curve,
    Close,
    Fill,
    Stroke,
    Layer,
    Clip,
    Tag,
    Text,
    Font,
    Image,
    Asset,
    Load,
    Style,
    Window,
    Title,
    Click,
    Play,
    Sound,
    Volume,
    Beep,
    Hover,
    Press,
    Drag,
    Mouse,
    Resize,
    Frame,
    Bind,
    State,
    Padding,
    Spacing,
    Row,
    Column,
    Scroll,
    Area,
    BtnClick,
    KeyDown,

    // Input Revolution
    Ask,
    Input,

    // Animation
    Step,
    Towards,
    Speed,
    SpeedVal(f64), // For 0.5x
    Rotate,
    Scale,
    Alpha,
    Tint,

    // Sbig (Web Server)
    Server,
    Sbig,
    Reply,
    Body,

    // Server Config
    Control,
    Workers,
    Limit,
    Per,
    Mins,
    Background,

    // Bit (Crypto)
    Bit,
    Aes,
    Code,
    Decode,
    Encrypt,
    Decrypt,
    Key,
    Iv,
    Nebc,
    Demon,

    // Maps (Dictionaries)
    Map,

    // PyBig
    PyBig,
    PythonCode(String),

    // Regex
    Bmath,

    // The Solver
    Solve,

    // Advanced Math
    Remainder,
    Sqrt,
    Sin,
    Cos,
    Tan,
    Abs,
    Floor,
    Ceil,
    Round,
    Log,
    Minimum,
    Maximum,
    PI,
    Euler,

    // String Modifiers
    Clean,
    Bigcap,
    Lower,
    Replace,

    // Logic Glue
    Smaller,
    Bigger,
    Between,
    Positive,
    Autolayering,

    // Markdown
    Markdown,

    // Environment
    Setting,

    // BigNet / BigWeb Power
    Task,
    Point,
    Note,
    SSL,
    Record,

    // Literals
    Identifier(String),
    Number(f64),
    String(String),

    // Symbols
    Assign,       // =
    Ampersand,    // &
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Caret,        // ^
    Greater,      // >
    Less,         // <
    GreaterEqual, // >=
    LessEqual,    // <=
    NotEqual,     // !=
    LBrace,       // {
    RBrace,       // }
    LParen,       // (
    RParen,       // )
    LBracket,     // [
    RBracket,     // ]
    Dot,          // .
    Colon,        // :
    Dollar,       // $
    At,           // @
    Char(char),   // Punctuation (! ? . etc)

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Token {
            token_type,
            line,
            column,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self.token_type, TokenType::Number(_))
    }
    pub fn is_operator(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Plus
                | TokenType::Minus
                | TokenType::Star
                | TokenType::Slash
                | TokenType::Caret
        )
    }
}
