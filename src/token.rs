use lexer::{is_digit, is_letter};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    ILLEGAL,
    EOF,
    IDENT(String),
    INT(String),
    ASSIGN,
    PLUS,
    MINUS,
    MULTIPLY,
    DIVIDE,
    LT,
    GT,
    BANG,
    COMMA,
    SEMICOLON,
    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,
    FUNCTION,
    LET,
    TRUE,
    FALSE,
    IF,
    ELSE,
    RETURN,
}

impl TokenType {
    pub fn from_str<'a>(s: &'a str) -> TokenType {
        match s {
            "=" => TokenType::ASSIGN,
            "+" => TokenType::PLUS,
            "-" => TokenType::MINUS,
            "*" => TokenType::MULTIPLY,
            "/" => TokenType::DIVIDE,
            "!" => TokenType::BANG,
            "<" => TokenType::LT,
            ">" => TokenType::GT,
            "," => TokenType::COMMA,
            ";" => TokenType::SEMICOLON,
            "(" => TokenType::LPAREN,
            ")" => TokenType::RPAREN,
            "{" => TokenType::LBRACE,
            "}" => TokenType::RBRACE,
            "let" => TokenType::LET,
            "fn" => TokenType::FUNCTION,
            "true" => TokenType::TRUE,
            "false" => TokenType::FALSE,
            "if" => TokenType::IF,
            "else" => TokenType::ELSE,
            "return" => TokenType::RETURN,
            "" => TokenType::EOF,
            n if is_digit(&n.to_string()) => TokenType::INT(n.to_string()),
            id if is_letter(&id.to_string()) => TokenType::IDENT(id.to_string()),
            _ => TokenType::ILLEGAL
        }
    }

    pub fn to_str<'a>(&'a self) -> String {
         (match *self {
             TokenType::EOF => "",
             TokenType::ASSIGN => "=",
             TokenType::PLUS => "+",
             TokenType::MINUS => "-",
             TokenType::MULTIPLY => "*",
             TokenType::DIVIDE => "/",
             TokenType::BANG => "!",
             TokenType::LT => "<",
             TokenType::GT => ">",
             TokenType::COMMA => ",",
             TokenType::SEMICOLON => ";",
             TokenType::LPAREN => "(",
             TokenType::RPAREN => ")",
             TokenType::LBRACE => "{",
             TokenType::RBRACE => "}",
             TokenType::FUNCTION => "fn",
             TokenType::LET => "let",
             TokenType::TRUE => "true",
             TokenType::FALSE => "false",
             TokenType::IF => "if",
             TokenType::ELSE => "else",
             TokenType::RETURN => "return",
             TokenType::INT(ref x) => x,
             TokenType::IDENT(ref x) => x,
             _ => "ILLEGAL",
         }).to_string()
    }
}

pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

pub fn new(s: String) -> Token {
    let tt = TokenType::from_str(s.as_str());

    Token {
        token_type: tt.clone(),
        literal: tt.to_str(),
    }
}
