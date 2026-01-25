//! HTML tokenizer
//!
//! Converts an HTML string into a stream of tokens. The tokenizer handles:
//! - Start tags with attributes
//! - End tags
//! - Self-closing tags
//! - Text content
//! - Comments
//! - Doctypes
//!
//! This is a simplified tokenizer focused on common patterns. It doesn't
//! implement the full HTML5 tokenization state machine but handles the
//! most common cases correctly.

use std::iter::Peekable;
use std::str::Chars;

/// An HTML token produced by the tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A DOCTYPE declaration.
    Doctype {
        /// The doctype name (usually "html").
        name: Option<String>,
    },
    /// A start tag.
    StartTag {
        /// The tag name (lowercase).
        name: String,
        /// Attribute key-value pairs.
        attributes: Vec<(String, String)>,
        /// Whether this is a self-closing tag (e.g., <br/>).
        self_closing: bool,
    },
    /// An end tag.
    EndTag {
        /// The tag name (lowercase).
        name: String,
    },
    /// Text content.
    Text(String),
    /// A comment.
    Comment(String),
}

/// HTML tokenizer that produces tokens from an HTML string.
pub struct Tokenizer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
    finished: bool,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer for the given HTML string.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            pos: 0,
            finished: false,
        }
    }

    /// Consumes the next character and advances position.
    fn consume(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Peeks at the next character without consuming it.
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Checks if the input starts with the given string at the current position.
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// Consumes characters while the predicate is true.
    fn consume_while<F: Fn(char) -> bool>(&mut self, pred: F) -> String {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if pred(c) {
                result.push(self.consume().unwrap());
            } else {
                break;
            }
        }
        result
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) {
        self.consume_while(|c| c.is_ascii_whitespace());
    }

    /// Parses a tag name.
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':')
            .to_ascii_lowercase()
    }

    /// Parses an attribute value (quoted or unquoted).
    fn parse_attribute_value(&mut self) -> String {
        self.skip_whitespace();

        match self.peek() {
            Some('"') => {
                self.consume(); // consume opening quote
                let value = self.consume_while(|c| c != '"');
                self.consume(); // consume closing quote
                value
            }
            Some('\'') => {
                self.consume(); // consume opening quote
                let value = self.consume_while(|c| c != '\'');
                self.consume(); // consume closing quote
                value
            }
            Some(_) => {
                // Unquoted attribute value
                self.consume_while(|c| !c.is_ascii_whitespace() && c != '>' && c != '/')
            }
            None => String::new(),
        }
    }

    /// Parses attributes from a start tag.
    fn parse_attributes(&mut self) -> Vec<(String, String)> {
        let mut attributes = Vec::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                None | Some('>') | Some('/') => break,
                Some(_) => {
                    // Parse attribute name
                    let name = self.consume_while(|c| {
                        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':'
                    })
                    .to_ascii_lowercase();

                    if name.is_empty() {
                        // Skip any unexpected character
                        self.consume();
                        continue;
                    }

                    self.skip_whitespace();

                    // Check for = sign
                    let value = if self.peek() == Some('=') {
                        self.consume(); // consume =
                        self.parse_attribute_value()
                    } else {
                        // Boolean attribute
                        String::new()
                    };

                    attributes.push((name, value));
                }
            }
        }

        attributes
    }

    /// Parses a start or end tag.
    fn parse_tag(&mut self) -> Token {
        self.consume(); // consume <

        // Check for special cases
        if self.starts_with("!--") {
            return self.parse_comment();
        }
        if self.starts_with("!DOCTYPE") || self.starts_with("!doctype") {
            return self.parse_doctype();
        }

        // Check for end tag
        let is_end_tag = self.peek() == Some('/');
        if is_end_tag {
            self.consume(); // consume /
        }

        let name = self.parse_tag_name();

        if is_end_tag {
            // Skip to closing >
            self.consume_while(|c| c != '>');
            self.consume(); // consume >
            return Token::EndTag { name };
        }

        // Parse attributes for start tag
        let attributes = self.parse_attributes();

        self.skip_whitespace();

        // Check for self-closing
        let self_closing = self.peek() == Some('/');
        if self_closing {
            self.consume(); // consume /
        }

        // Consume closing >
        if self.peek() == Some('>') {
            self.consume();
        }

        Token::StartTag {
            name,
            attributes,
            self_closing,
        }
    }

    /// Parses a comment.
    fn parse_comment(&mut self) -> Token {
        // Consume !--
        self.consume(); // !
        self.consume(); // -
        self.consume(); // -

        let mut content = String::new();

        loop {
            if self.starts_with("-->") {
                self.consume(); // -
                self.consume(); // -
                self.consume(); // >
                break;
            }
            match self.consume() {
                Some(c) => content.push(c),
                None => break,
            }
        }

        Token::Comment(content)
    }

    /// Parses a DOCTYPE.
    fn parse_doctype(&mut self) -> Token {
        // Consume !DOCTYPE or !doctype
        self.consume_while(|c| c.is_ascii_alphabetic() || c == '!');
        self.skip_whitespace();

        let name = if self.peek().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
            Some(self.parse_tag_name())
        } else {
            None
        };

        // Skip to closing >
        self.consume_while(|c| c != '>');
        self.consume(); // consume >

        Token::Doctype { name }
    }

    /// Parses text content.
    fn parse_text(&mut self) -> Token {
        let text = self.consume_while(|c| c != '<');
        Token::Text(text)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.peek() {
            None => {
                self.finished = true;
                None
            }
            Some('<') => Some(self.parse_tag()),
            Some(_) => {
                let token = self.parse_text();
                // Don't emit empty text tokens
                if let Token::Text(ref text) = token {
                    if text.is_empty() {
                        return self.next();
                    }
                }
                Some(token)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_element() {
        let tokens: Vec<_> = Tokenizer::new("<div></div>").collect();
        assert_eq!(tokens, vec![
            Token::StartTag {
                name: "div".to_string(),
                attributes: vec![],
                self_closing: false,
            },
            Token::EndTag {
                name: "div".to_string(),
            },
        ]);
    }

    #[test]
    fn test_text_content() {
        let tokens: Vec<_> = Tokenizer::new("<p>Hello World</p>").collect();
        assert_eq!(tokens, vec![
            Token::StartTag {
                name: "p".to_string(),
                attributes: vec![],
                self_closing: false,
            },
            Token::Text("Hello World".to_string()),
            Token::EndTag {
                name: "p".to_string(),
            },
        ]);
    }

    #[test]
    fn test_attributes() {
        let tokens: Vec<_> = Tokenizer::new(r#"<div class="container" id="main"></div>"#).collect();
        assert_eq!(tokens, vec![
            Token::StartTag {
                name: "div".to_string(),
                attributes: vec![
                    ("class".to_string(), "container".to_string()),
                    ("id".to_string(), "main".to_string()),
                ],
                self_closing: false,
            },
            Token::EndTag {
                name: "div".to_string(),
            },
        ]);
    }

    #[test]
    fn test_self_closing() {
        let tokens: Vec<_> = Tokenizer::new("<br/>").collect();
        assert_eq!(tokens, vec![
            Token::StartTag {
                name: "br".to_string(),
                attributes: vec![],
                self_closing: true,
            },
        ]);
    }

    #[test]
    fn test_comment() {
        let tokens: Vec<_> = Tokenizer::new("<!-- This is a comment -->").collect();
        assert_eq!(tokens, vec![
            Token::Comment(" This is a comment ".to_string()),
        ]);
    }

    #[test]
    fn test_doctype() {
        let tokens: Vec<_> = Tokenizer::new("<!DOCTYPE html>").collect();
        assert_eq!(tokens, vec![
            Token::Doctype {
                name: Some("html".to_string()),
            },
        ]);
    }

    #[test]
    fn test_lowercase_tags() {
        let tokens: Vec<_> = Tokenizer::new("<DIV></DIV>").collect();
        assert_eq!(tokens, vec![
            Token::StartTag {
                name: "div".to_string(),
                attributes: vec![],
                self_closing: false,
            },
            Token::EndTag {
                name: "div".to_string(),
            },
        ]);
    }

    #[test]
    fn test_boolean_attribute() {
        let tokens: Vec<_> = Tokenizer::new("<input disabled>").collect();
        assert_eq!(tokens, vec![
            Token::StartTag {
                name: "input".to_string(),
                attributes: vec![
                    ("disabled".to_string(), String::new()),
                ],
                self_closing: false,
            },
        ]);
    }

    #[test]
    fn test_unquoted_attribute() {
        let tokens: Vec<_> = Tokenizer::new("<div class=container>").collect();
        assert_eq!(tokens, vec![
            Token::StartTag {
                name: "div".to_string(),
                attributes: vec![
                    ("class".to_string(), "container".to_string()),
                ],
                self_closing: false,
            },
        ]);
    }

    #[test]
    fn test_single_quoted_attribute() {
        let tokens: Vec<_> = Tokenizer::new("<div class='container'>").collect();
        assert_eq!(tokens, vec![
            Token::StartTag {
                name: "div".to_string(),
                attributes: vec![
                    ("class".to_string(), "container".to_string()),
                ],
                self_closing: false,
            },
        ]);
    }

    #[test]
    fn test_nested_elements() {
        let tokens: Vec<_> = Tokenizer::new("<div><p>Hello</p></div>").collect();
        assert_eq!(tokens, vec![
            Token::StartTag { name: "div".to_string(), attributes: vec![], self_closing: false },
            Token::StartTag { name: "p".to_string(), attributes: vec![], self_closing: false },
            Token::Text("Hello".to_string()),
            Token::EndTag { name: "p".to_string() },
            Token::EndTag { name: "div".to_string() },
        ]);
    }
}
