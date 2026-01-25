//! CSS Tokenizer
//!
//! Lexical analysis of CSS text into tokens. This is a simplified tokenizer
//! that handles common CSS patterns without full CSS3 specification compliance.

use std::iter::Peekable;
use std::str::Chars;

/// A CSS token with its position in the source
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

/// The kind of CSS token
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// An identifier (property name, tag name, etc.)
    Ident(String),
    /// A hash token (#id or #color)
    Hash(String),
    /// A string literal ("..." or '...')
    String(String),
    /// A number (integer or float)
    Number(f64),
    /// A dimension (number with unit, e.g., 10px)
    Dimension(f64, String),
    /// A percentage (e.g., 50%)
    Percentage(f64),
    /// A function name followed by (
    Function(String),
    /// @-keyword (e.g., @import, @media)
    AtKeyword(String),
    /// Colon :
    Colon,
    /// Semicolon ;
    Semicolon,
    /// Comma ,
    Comma,
    /// Left brace {
    LeftBrace,
    /// Right brace }
    RightBrace,
    /// Left bracket [
    LeftBracket,
    /// Right bracket ]
    RightBracket,
    /// Left parenthesis (
    LeftParen,
    /// Right parenthesis )
    RightParen,
    /// Greater than >
    GreaterThan,
    /// Plus +
    Plus,
    /// Tilde ~
    Tilde,
    /// Asterisk *
    Asterisk,
    /// Period .
    Period,
    /// Equals =
    Equals,
    /// Pipe |
    Pipe,
    /// Caret ^
    Caret,
    /// Dollar $
    Dollar,
    /// Exclamation !
    Exclamation,
    /// Whitespace (collapsed)
    Whitespace,
    /// End of file
    Eof,
    /// Any other single character
    Delim(char),
}

/// Iterator-based CSS tokenizer
pub struct Tokenizer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer for the given CSS input
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            input,
            chars: input.chars().peekable(),
            pos: 0,
        }
    }

    /// Peek at the next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Consume and return the next character
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    /// Skip whitespace and return true if any was skipped
    fn skip_whitespace(&mut self) -> bool {
        let mut skipped = false;
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
                skipped = true;
            } else {
                break;
            }
        }
        skipped
    }

    /// Skip a comment (/* ... */)
    fn skip_comment(&mut self) -> bool {
        if self.peek() == Some('/') {
            let start_pos = self.pos;
            self.advance();
            if self.peek() == Some('*') {
                self.advance();
                // Find the end of the comment
                loop {
                    match self.advance() {
                        Some('*') if self.peek() == Some('/') => {
                            self.advance();
                            return true;
                        }
                        None => return true, // Unterminated comment
                        _ => continue,
                    }
                }
            } else {
                // Not a comment, restore position
                // Note: We can't truly restore, so we'll handle this differently
                // by checking for /* before calling this
                self.pos = start_pos;
                return false;
            }
        }
        false
    }

    /// Check if the next two characters are /*
    fn is_comment_start(&mut self) -> bool {
        if self.peek() == Some('/') {
            // We need to look ahead two characters
            let mut chars = self.input[self.pos..].chars();
            chars.next(); // Skip the first /
            chars.next() == Some('*')
        } else {
            false
        }
    }

    /// Skip all whitespace and comments
    fn skip_whitespace_and_comments(&mut self) -> bool {
        let mut skipped = false;
        loop {
            let found_whitespace = self.skip_whitespace();
            let found_comment = self.is_comment_start() && self.skip_comment();

            if found_whitespace || found_comment {
                skipped = true;
            } else {
                break;
            }
        }
        skipped
    }

    /// Read an identifier
    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.peek() {
            if is_ident_char(c) {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    /// Read a number (integer or float)
    fn read_number(&mut self) -> f64 {
        let mut num_str = String::new();

        // Handle negative sign
        if self.peek() == Some('-') {
            num_str.push('-');
            self.advance();
        }

        // Integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Decimal part
        if self.peek() == Some('.') {
            num_str.push('.');
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        num_str.parse().unwrap_or(0.0)
    }

    /// Read a string literal
    fn read_string(&mut self, quote: char) -> String {
        let mut s = String::new();
        self.advance(); // Consume opening quote

        while let Some(c) = self.advance() {
            if c == quote {
                break;
            } else if c == '\\' {
                // Handle escape sequences
                if let Some(escaped) = self.advance() {
                    match escaped {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        _ => s.push(escaped),
                    }
                }
            } else {
                s.push(c);
            }
        }

        s
    }

    /// Read a hash token (ID selector or color)
    fn read_hash(&mut self) -> String {
        self.advance(); // Consume #
        self.read_ident()
    }

    /// Read an at-keyword
    fn read_at_keyword(&mut self) -> String {
        self.advance(); // Consume @
        self.read_ident()
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Token {
        let had_whitespace = self.skip_whitespace_and_comments();

        let start = self.pos;

        // Return whitespace token if we had significant whitespace
        // (We collapse all whitespace into a single token for simplicity,
        // but only emit it when it's between other tokens)
        if had_whitespace
            && self.peek().is_some()
            && !matches!(self.peek(), Some('{') | Some('}') | Some(';'))
        {
            return Token {
                kind: TokenKind::Whitespace,
                start,
                end: self.pos,
            };
        }

        // Re-skip after potentially emitting whitespace
        self.skip_whitespace_and_comments();
        let start = self.pos;

        let kind = match self.peek() {
            None => TokenKind::Eof,
            Some(c) => match c {
                ':' => {
                    self.advance();
                    TokenKind::Colon
                }
                ';' => {
                    self.advance();
                    TokenKind::Semicolon
                }
                ',' => {
                    self.advance();
                    TokenKind::Comma
                }
                '{' => {
                    self.advance();
                    TokenKind::LeftBrace
                }
                '}' => {
                    self.advance();
                    TokenKind::RightBrace
                }
                '[' => {
                    self.advance();
                    TokenKind::LeftBracket
                }
                ']' => {
                    self.advance();
                    TokenKind::RightBracket
                }
                '(' => {
                    self.advance();
                    TokenKind::LeftParen
                }
                ')' => {
                    self.advance();
                    TokenKind::RightParen
                }
                '>' => {
                    self.advance();
                    TokenKind::GreaterThan
                }
                '+' => {
                    self.advance();
                    TokenKind::Plus
                }
                '~' => {
                    self.advance();
                    TokenKind::Tilde
                }
                '*' => {
                    self.advance();
                    TokenKind::Asterisk
                }
                '.' => {
                    self.advance();
                    TokenKind::Period
                }
                '=' => {
                    self.advance();
                    TokenKind::Equals
                }
                '|' => {
                    self.advance();
                    TokenKind::Pipe
                }
                '^' => {
                    self.advance();
                    TokenKind::Caret
                }
                '$' => {
                    self.advance();
                    TokenKind::Dollar
                }
                '!' => {
                    self.advance();
                    TokenKind::Exclamation
                }
                '#' => {
                    let hash = self.read_hash();
                    TokenKind::Hash(hash)
                }
                '@' => {
                    let keyword = self.read_at_keyword();
                    TokenKind::AtKeyword(keyword)
                }
                '"' | '\'' => {
                    let s = self.read_string(c);
                    TokenKind::String(s)
                }
                c if c.is_ascii_digit() || (c == '-' && self.is_number_start()) => {
                    let num = self.read_number();

                    // Check for unit or percentage
                    if self.peek() == Some('%') {
                        self.advance();
                        TokenKind::Percentage(num)
                    } else if let Some(c) = self.peek() {
                        if is_ident_start(c) {
                            let unit = self.read_ident();
                            TokenKind::Dimension(num, unit)
                        } else {
                            TokenKind::Number(num)
                        }
                    } else {
                        TokenKind::Number(num)
                    }
                }
                c if is_ident_start(c) => {
                    let ident = self.read_ident();
                    // Check if it's a function
                    if self.peek() == Some('(') {
                        TokenKind::Function(ident)
                    } else {
                        TokenKind::Ident(ident)
                    }
                }
                '-' => {
                    // Could be a negative number or an identifier starting with -
                    if self.is_number_start() {
                        let num = self.read_number();
                        if self.peek() == Some('%') {
                            self.advance();
                            TokenKind::Percentage(num)
                        } else if let Some(c) = self.peek() {
                            if is_ident_start(c) {
                                let unit = self.read_ident();
                                TokenKind::Dimension(num, unit)
                            } else {
                                TokenKind::Number(num)
                            }
                        } else {
                            TokenKind::Number(num)
                        }
                    } else {
                        // Identifier starting with -
                        let ident = self.read_ident();
                        if ident.is_empty() {
                            self.advance();
                            TokenKind::Delim('-')
                        } else if self.peek() == Some('(') {
                            TokenKind::Function(ident)
                        } else {
                            TokenKind::Ident(ident)
                        }
                    }
                }
                _ => {
                    self.advance();
                    TokenKind::Delim(c)
                }
            },
        };

        Token {
            kind,
            start,
            end: self.pos,
        }
    }

    /// Check if the next characters form a number
    fn is_number_start(&mut self) -> bool {
        let rest = &self.input[self.pos..];
        let mut chars = rest.chars();

        match chars.next() {
            Some('-') => matches!(chars.next(), Some(c) if c.is_ascii_digit() || c == '.'),
            Some(c) if c.is_ascii_digit() => true,
            Some('.') => matches!(chars.next(), Some(c) if c.is_ascii_digit()),
            _ => false,
        }
    }

    /// Collect all tokens into a vector
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

/// Check if a character can start an identifier
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '-'
}

/// Check if a character can be part of an identifier
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<TokenKind> {
        Tokenizer::new(input)
            .tokenize()
            .into_iter()
            .map(|t| t.kind)
            .filter(|k| !matches!(k, TokenKind::Whitespace))
            .collect()
    }

    #[test]
    fn tokenize_ident() {
        let tokens = tokenize("body");
        assert_eq!(
            tokens,
            vec![TokenKind::Ident("body".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenize_hash() {
        let tokens = tokenize("#main");
        assert_eq!(
            tokens,
            vec![TokenKind::Hash("main".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenize_class() {
        let tokens = tokenize(".container");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Period,
                TokenKind::Ident("container".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_number() {
        let tokens = tokenize("42");
        assert_eq!(tokens, vec![TokenKind::Number(42.0), TokenKind::Eof]);
    }

    #[test]
    fn tokenize_float() {
        let tokens = tokenize("3.5");
        assert_eq!(tokens, vec![TokenKind::Number(3.5), TokenKind::Eof]);
    }

    #[test]
    fn tokenize_dimension() {
        let tokens = tokenize("10px");
        assert_eq!(
            tokens,
            vec![TokenKind::Dimension(10.0, "px".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenize_percentage() {
        let tokens = tokenize("50%");
        assert_eq!(tokens, vec![TokenKind::Percentage(50.0), TokenKind::Eof]);
    }

    #[test]
    fn tokenize_string_double() {
        let tokens = tokenize(r#""hello""#);
        assert_eq!(
            tokens,
            vec![TokenKind::String("hello".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenize_string_single() {
        let tokens = tokenize("'world'");
        assert_eq!(
            tokens,
            vec![TokenKind::String("world".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenize_function() {
        let tokens = tokenize("rgb(");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Function("rgb".to_string()),
                TokenKind::LeftParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_at_keyword() {
        let tokens = tokenize("@media");
        assert_eq!(
            tokens,
            vec![TokenKind::AtKeyword("media".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenize_simple_rule() {
        let tokens = tokenize("body { color: red; }");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ident("body".to_string()),
                TokenKind::LeftBrace,
                TokenKind::Ident("color".to_string()),
                TokenKind::Colon,
                TokenKind::Ident("red".to_string()),
                TokenKind::Semicolon,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_hex_color() {
        let tokens = tokenize("#ff0000");
        assert_eq!(
            tokens,
            vec![TokenKind::Hash("ff0000".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenize_comment() {
        let tokens = tokenize("/* comment */ body");
        assert_eq!(
            tokens,
            vec![TokenKind::Ident("body".to_string()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenize_negative_number() {
        let tokens = tokenize("-10px");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Dimension(-10.0, "px".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn tokenize_vendor_prefix() {
        let tokens = tokenize("-webkit-transform");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ident("-webkit-transform".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn tokenize_combinators() {
        let tokens = tokenize("div > p + span ~ a");
        let expected = vec![
            TokenKind::Ident("div".to_string()),
            TokenKind::GreaterThan,
            TokenKind::Ident("p".to_string()),
            TokenKind::Plus,
            TokenKind::Ident("span".to_string()),
            TokenKind::Tilde,
            TokenKind::Ident("a".to_string()),
            TokenKind::Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn tokenize_attribute_selector() {
        let tokens = tokenize("[disabled]");
        assert_eq!(
            tokens,
            vec![
                TokenKind::LeftBracket,
                TokenKind::Ident("disabled".to_string()),
                TokenKind::RightBracket,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_important() {
        let tokens = tokenize("!important");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Exclamation,
                TokenKind::Ident("important".to_string()),
                TokenKind::Eof,
            ]
        );
    }
}
