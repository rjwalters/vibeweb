//! HTML tokenizer.
//!
//! Converts HTML source text into a stream of tokens for the parser.

use std::collections::HashMap;

/// A token produced by the HTML tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// DOCTYPE declaration
    DocType(String),
    /// Start tag (tag name, attributes, self-closing)
    StartTag {
        name: String,
        attributes: HashMap<String, String>,
        self_closing: bool,
    },
    /// End tag
    EndTag(String),
    /// Text content
    Text(String),
    /// Comment
    Comment(String),
    /// End of input
    Eof,
}

/// HTML tokenizer state machine.
pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer for the given input.
    pub fn new(input: &'a str) -> Self {
        Tokenizer { input, pos: 0 }
    }

    /// Returns the next token from the input.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_between_tags();

        if self.pos >= self.input.len() {
            return Token::Eof;
        }

        if self.starts_with("<!--") {
            return self.consume_comment();
        }

        if self.starts_with("<!DOCTYPE") || self.starts_with("<!doctype") {
            return self.consume_doctype();
        }

        if self.starts_with("</") {
            return self.consume_end_tag();
        }

        if self.starts_with("<") {
            return self.consume_start_tag();
        }

        self.consume_text()
    }

    /// Peeks at the current character without consuming it.
    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Consumes and returns the current character.
    fn consume_char(&mut self) -> Option<char> {
        let c = self.current_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Checks if the remaining input starts with the given string.
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// Skips whitespace only if we're between tags (not inside text content).
    fn skip_whitespace_between_tags(&mut self) {
        // Only skip whitespace if the next non-whitespace is a '<'
        let remaining = &self.input[self.pos..];
        let trimmed = remaining.trim_start();
        if trimmed.starts_with('<') {
            self.pos = self.input.len() - trimmed.len();
        }
    }

    /// Consumes a DOCTYPE declaration.
    fn consume_doctype(&mut self) -> Token {
        // Skip "<!DOCTYPE" or "<!doctype"
        self.pos += 9;
        self.skip_whitespace();

        let name = self.consume_while(|c| c != '>');
        if self.current_char() == Some('>') {
            self.consume_char();
        }

        Token::DocType(name.trim().to_string())
    }

    /// Consumes a comment.
    fn consume_comment(&mut self) -> Token {
        // Skip "<!--"
        self.pos += 4;

        let mut content = String::new();
        while !self.starts_with("-->") && self.pos < self.input.len() {
            if let Some(c) = self.consume_char() {
                content.push(c);
            }
        }

        // Skip "-->"
        if self.starts_with("-->") {
            self.pos += 3;
        }

        Token::Comment(content)
    }

    /// Consumes a start tag.
    fn consume_start_tag(&mut self) -> Token {
        // Skip "<"
        self.consume_char();
        self.skip_whitespace();

        let name = self.consume_tag_name();
        let attributes = self.consume_attributes();

        self.skip_whitespace();
        let self_closing = self.starts_with("/>");

        if self_closing {
            self.pos += 2;
        } else if self.current_char() == Some('>') {
            self.consume_char();
        }

        Token::StartTag {
            name: name.to_lowercase(),
            attributes,
            self_closing,
        }
    }

    /// Consumes an end tag.
    fn consume_end_tag(&mut self) -> Token {
        // Skip "</"
        self.pos += 2;
        self.skip_whitespace();

        let name = self.consume_tag_name();

        // Skip to closing >
        while self.current_char() != Some('>') && self.pos < self.input.len() {
            self.consume_char();
        }
        if self.current_char() == Some('>') {
            self.consume_char();
        }

        Token::EndTag(name.to_lowercase())
    }

    /// Consumes text content until we hit a tag.
    fn consume_text(&mut self) -> Token {
        let text = self.consume_while(|c| c != '<');
        Token::Text(text)
    }

    /// Consumes a tag name.
    fn consume_tag_name(&mut self) -> String {
        self.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
    }

    /// Consumes all attributes in a tag.
    fn consume_attributes(&mut self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();

        loop {
            self.skip_whitespace();

            match self.current_char() {
                Some('>') | Some('/') | None => break,
                _ => {
                    if let Some((name, value)) = self.consume_attribute() {
                        attrs.insert(name, value);
                    }
                }
            }
        }

        attrs
    }

    /// Consumes a single attribute.
    fn consume_attribute(&mut self) -> Option<(String, String)> {
        let name = self.consume_while(|c| {
            c.is_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.'
        });

        if name.is_empty() {
            return None;
        }

        self.skip_whitespace();

        // Check for = sign
        if self.current_char() != Some('=') {
            // Boolean attribute
            return Some((name.to_lowercase(), String::new()));
        }

        // Skip =
        self.consume_char();
        self.skip_whitespace();

        let value = match self.current_char() {
            Some('"') => {
                self.consume_char(); // Skip opening quote
                let v = self.consume_while(|c| c != '"');
                self.consume_char(); // Skip closing quote
                v
            }
            Some('\'') => {
                self.consume_char(); // Skip opening quote
                let v = self.consume_while(|c| c != '\'');
                self.consume_char(); // Skip closing quote
                v
            }
            _ => {
                // Unquoted attribute value
                self.consume_while(|c| !c.is_whitespace() && c != '>' && c != '/')
            }
        };

        Some((name.to_lowercase(), value))
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    /// Consumes characters while the predicate is true.
    fn consume_while<F>(&mut self, pred: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(c) = self.current_char() {
            if pred(c) {
                result.push(c);
                self.consume_char();
            } else {
                break;
            }
        }
        result
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Token::Eof => None,
            token => Some(token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let mut tokenizer = Tokenizer::new("<div></div>");
        assert!(matches!(
            tokenizer.next_token(),
            Token::StartTag { name, .. } if name == "div"
        ));
        assert!(matches!(tokenizer.next_token(), Token::EndTag(name) if name == "div"));
        assert!(matches!(tokenizer.next_token(), Token::Eof));
    }

    #[test]
    fn test_text_content() {
        let mut tokenizer = Tokenizer::new("<p>Hello World</p>");
        tokenizer.next_token(); // <p>
        let text = tokenizer.next_token();
        assert!(matches!(text, Token::Text(t) if t == "Hello World"));
    }

    #[test]
    fn test_attributes() {
        let mut tokenizer = Tokenizer::new("<div class=\"foo\" id='bar'></div>");
        if let Token::StartTag { attributes, .. } = tokenizer.next_token() {
            assert_eq!(attributes.get("class"), Some(&"foo".to_string()));
            assert_eq!(attributes.get("id"), Some(&"bar".to_string()));
        } else {
            panic!("Expected start tag");
        }
    }

    #[test]
    fn test_self_closing() {
        let mut tokenizer = Tokenizer::new("<br/>");
        if let Token::StartTag { self_closing, .. } = tokenizer.next_token() {
            assert!(self_closing);
        } else {
            panic!("Expected start tag");
        }
    }

    #[test]
    fn test_doctype() {
        let mut tokenizer = Tokenizer::new("<!DOCTYPE html>");
        assert!(matches!(tokenizer.next_token(), Token::DocType(name) if name == "html"));
    }

    #[test]
    fn test_comment() {
        let mut tokenizer = Tokenizer::new("<!-- This is a comment -->");
        assert!(matches!(
            tokenizer.next_token(),
            Token::Comment(text) if text.trim() == "This is a comment"
        ));
    }

    #[test]
    fn test_case_insensitive_tags() {
        let mut tokenizer = Tokenizer::new("<DIV></DIV>");
        assert!(matches!(
            tokenizer.next_token(),
            Token::StartTag { name, .. } if name == "div"
        ));
        assert!(matches!(tokenizer.next_token(), Token::EndTag(name) if name == "div"));
    }
}
