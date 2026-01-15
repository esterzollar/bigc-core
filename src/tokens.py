from enum import Enum, auto

class TokenType(Enum):
    # Keywords
    DEFINATION = auto() # #defination or just context
    ITEM = auto()       # item_X
    GET = auto()
    SET = auto()
    AS = auto()
    PRINT = auto()
    IF = auto()
    ELSE = auto()
    TAKE = auto()
    WAIT = auto()
    TYPE = auto()
    START = auto()
    RESET = auto()
    KEEP = auto()
    ADDRUN = auto()
    ANDRUN = auto()
    END = auto()
    ANY = auto()
    BUG = auto()
    FOUND = auto()

    # Literals
    NUMBER = auto()
    STRING = auto()
    IDENTIFIER = auto()
    
    # Operators & Symbols
    ASSIGN = auto()     # =
    AMPERSAND = auto()  # &
    DOLLAR = auto()     # $
    PLUS = auto()       # +
    MINUS = auto()      # -
    MULTIPLY = auto()   # *
    DIVIDE = auto()     # /
    GT = auto()         # >
    LT = auto()         # <
    LBRACE = auto()     # {
    RBRACE = auto()     # }
    UNDERSCORE = auto() # _ (for _output)
    
    EOF = auto()

class Token:
    def __init__(self, type, value=None, line=1):
        self.type = type
        self.value = value
        self.line = line

    def __repr__(self):
        return f"Token({self.type.name}, {repr(self.value)})"
