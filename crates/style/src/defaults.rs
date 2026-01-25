//! User Agent Stylesheet and Default Styles.
//!
//! This module provides browser default styles for HTML elements,
//! implementing a minimal user-agent stylesheet.

use crate::cascade::{CssValue, Declaration, MatchedRule, Origin, PropertyId, Specificity};
use crate::properties::Display;
use crate::values::{Length, LengthOrAuto};

/// Get the default display value for an HTML element.
pub fn default_display_for_element(tag_name: &str) -> Display {
    match tag_name.to_lowercase().as_str() {
        // Block-level elements
        "html" | "body" | "div" | "article" | "section" | "nav" | "aside" | "header" | "footer"
        | "main" | "address" | "blockquote" | "figure" | "figcaption" | "hgroup" | "search"
        | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "pre" | "hr" | "form" | "fieldset"
        | "legend" | "ol" | "ul" | "li" | "dl" | "dt" | "dd" | "table" | "caption"
        | "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" | "colgroup" | "col"
        | "details" | "summary" | "dialog" | "menu" | "dir" | "center" | "noscript"
        | "output" | "video" | "audio" | "canvas" | "map" | "object" | "iframe"
        | "frameset" | "frame" => Display::Block,

        // Inline elements
        "span" | "a" | "abbr" | "acronym" | "b" | "bdi" | "bdo" | "big" | "br" | "cite"
        | "code" | "data" | "del" | "dfn" | "em" | "i" | "ins" | "kbd" | "mark" | "meter"
        | "progress" | "q" | "rb" | "rp" | "rt" | "rtc" | "ruby" | "s" | "samp" | "small"
        | "strike" | "strong" | "sub" | "sup" | "time" | "tt" | "u" | "var" | "wbr"
        | "font" | "nobr" => Display::Inline,

        // Inline-block elements
        "button" | "input" | "select" | "textarea" | "img" => Display::InlineBlock,

        // Hidden elements
        "head" | "title" | "meta" | "link" | "style" | "script" | "base" | "template"
        | "noembed" | "param" | "source" | "track" | "area" | "datalist" | "slot" => Display::None,

        // Default to inline for unknown elements (per HTML spec)
        _ => Display::Inline,
    }
}

/// Get the user-agent stylesheet rules for an element.
///
/// Returns a list of matched rules with UA origin and zero specificity.
pub fn user_agent_rules_for_element(tag_name: &str) -> Vec<MatchedRule> {
    let mut rules = Vec::new();
    let mut declarations = Vec::new();

    let tag = tag_name.to_lowercase();

    // Display property
    let display = default_display_for_element(&tag);
    declarations.push(Declaration::new(
        PropertyId::Display,
        CssValue::Keyword(display.to_string()),
    ));

    // Element-specific defaults
    match tag.as_str() {
        // Headings
        "h1" => {
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(2.0)),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontWeight,
                CssValue::Keyword("bold".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(0.67))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(0.67))),
            ));
        }
        "h2" => {
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(1.5)),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontWeight,
                CssValue::Keyword("bold".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(0.83))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(0.83))),
            ));
        }
        "h3" => {
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(1.17)),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontWeight,
                CssValue::Keyword("bold".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
        }
        "h4" => {
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(1.0)),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontWeight,
                CssValue::Keyword("bold".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.33))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.33))),
            ));
        }
        "h5" => {
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(0.83)),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontWeight,
                CssValue::Keyword("bold".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.67))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.67))),
            ));
        }
        "h6" => {
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(0.67)),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontWeight,
                CssValue::Keyword("bold".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(2.33))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(2.33))),
            ));
        }

        // Paragraph
        "p" => {
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
        }

        // Body
        "body" => {
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::px(8.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginRight,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::px(8.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::px(8.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginLeft,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::px(8.0))),
            ));
        }

        // Lists
        "ul" | "ol" | "menu" | "dir" => {
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::PaddingLeft,
                CssValue::Length(Length::px(40.0)),
            ));
        }

        // Blockquote
        "blockquote" => {
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginLeft,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::px(40.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginRight,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::px(40.0))),
            ));
        }

        // Pre
        "pre" => {
            declarations.push(Declaration::new(
                PropertyId::WhiteSpace,
                CssValue::Keyword("pre".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontFamily,
                CssValue::String("monospace".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(1.0))),
            ));
        }

        // Code, kbd, samp, tt
        "code" | "kbd" | "samp" | "tt" => {
            declarations.push(Declaration::new(
                PropertyId::FontFamily,
                CssValue::String("monospace".to_string()),
            ));
        }

        // Strong, b
        "strong" | "b" => {
            declarations.push(Declaration::new(
                PropertyId::FontWeight,
                CssValue::Keyword("bold".to_string()),
            ));
        }

        // Em, i, cite, var, dfn
        "em" | "i" | "cite" | "var" | "dfn" => {
            // Would need font-style: italic, but we don't support that yet
        }

        // Small
        "small" => {
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(0.833)),
            ));
        }

        // Big
        "big" => {
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(1.17)),
            ));
        }

        // Sub
        "sub" => {
            declarations.push(Declaration::new(
                PropertyId::VerticalAlign,
                CssValue::Keyword("sub".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(0.833)),
            ));
        }

        // Sup
        "sup" => {
            declarations.push(Declaration::new(
                PropertyId::VerticalAlign,
                CssValue::Keyword("super".to_string()),
            ));
            declarations.push(Declaration::new(
                PropertyId::FontSize,
                CssValue::Length(Length::em(0.833)),
            ));
        }

        // Horizontal rule
        "hr" => {
            declarations.push(Declaration::new(
                PropertyId::MarginTop,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(0.5))),
            ));
            declarations.push(Declaration::new(
                PropertyId::MarginBottom,
                CssValue::LengthOrAuto(LengthOrAuto::Length(Length::em(0.5))),
            ));
            declarations.push(Declaration::new(
                PropertyId::BorderTopWidth,
                CssValue::Length(Length::px(1.0)),
            ));
        }

        // Address
        "address" => {
            // Would need font-style: italic
        }

        // Center (deprecated but still used)
        "center" => {
            declarations.push(Declaration::new(
                PropertyId::TextAlign,
                CssValue::Keyword("center".to_string()),
            ));
        }

        _ => {}
    }

    if !declarations.is_empty() {
        rules.push(MatchedRule::new(
            declarations,
            Specificity::zero(),
            Origin::UserAgent,
            0,
        ));
    }

    rules
}

/// Check if an element is a phrasing content element (inline context).
pub fn is_phrasing_content(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "a" | "abbr" | "area" | "audio" | "b" | "bdi" | "bdo" | "br" | "button" | "canvas"
            | "cite" | "code" | "data" | "datalist" | "del" | "dfn" | "em" | "embed" | "i"
            | "iframe" | "img" | "input" | "ins" | "kbd" | "label" | "map" | "mark" | "math"
            | "meter" | "noscript" | "object" | "output" | "picture" | "progress" | "q"
            | "ruby" | "s" | "samp" | "script" | "select" | "slot" | "small" | "span"
            | "strong" | "sub" | "sup" | "svg" | "template" | "textarea" | "time" | "u"
            | "var" | "video" | "wbr"
    )
}

/// Check if an element is a heading element.
pub fn is_heading(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
    )
}

/// Check if an element is a sectioning element.
pub fn is_sectioning(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "article" | "aside" | "nav" | "section"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::Display;

    #[test]
    fn test_default_display() {
        assert_eq!(default_display_for_element("div"), Display::Block);
        assert_eq!(default_display_for_element("span"), Display::Inline);
        assert_eq!(default_display_for_element("button"), Display::InlineBlock);
        assert_eq!(default_display_for_element("head"), Display::None);
        assert_eq!(default_display_for_element("custom-element"), Display::Inline);
    }

    #[test]
    fn test_user_agent_rules() {
        let h1_rules = user_agent_rules_for_element("h1");
        assert!(!h1_rules.is_empty());

        let rule = &h1_rules[0];
        assert_eq!(rule.origin, Origin::UserAgent);
        assert_eq!(rule.specificity, Specificity::zero());

        // Should have display, font-size, font-weight, margins
        assert!(rule.declarations.len() >= 4);
    }

    #[test]
    fn test_phrasing_content() {
        assert!(is_phrasing_content("span"));
        assert!(is_phrasing_content("a"));
        assert!(!is_phrasing_content("div"));
        assert!(!is_phrasing_content("p"));
    }

    #[test]
    fn test_heading_detection() {
        assert!(is_heading("h1"));
        assert!(is_heading("H6"));
        assert!(!is_heading("p"));
        assert!(!is_heading("header"));
    }
}
