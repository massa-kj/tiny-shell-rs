#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Ident,         // Command name or variable name
    String,        // Quoted string
    Number,        // Number
    Assign,        // =
    Pipe,          // |
    And,           // &&
    Or,            // ||
    RedirectIn,    // <
    RedirectOut,   // >
    RedirectAppend,// >>
    RedirectErr,   // 2>
    Semicolon,     // ;
    Amp,           // &
    LParen,        // (
    RParen,        // )
    LBrace,        // {
    RBrace,        // }
    If, Then, Else, Fi, For, While, Do, Done, // Keywords
    Dollar,        // $
    Backtick,      // `
    SubstitutionStart, // $(
    SubstitutionEnd,   // )
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,   // Original string
    pub span: (usize, usize), // Position info [start, end)
}

