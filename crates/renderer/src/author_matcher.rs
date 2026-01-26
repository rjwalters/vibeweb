//! Author stylesheet matcher for CSS rule matching.
//!
//! This module provides the `AuthorStyleMatcher` which implements the `RuleMatcher`
//! trait from vw-style. It uses the vw-css selector matching engine to match
//! author stylesheet rules against DOM elements.

use vw_css::{Stylesheet, matching::matches, specificity};
use vw_dom::{Document as DomDocument, NodeId as DomNodeId};
use vw_style::tree::{RuleMatcher, NodeInfo};
use vw_style::cascade::{MatchedRule, Declaration, Origin, PropertyId, CssValue};
use vw_gfx::color::Color;

/// A rule matcher that applies author stylesheets using CSS selector matching.
///
/// This matcher combines User-Agent defaults with author stylesheet rules,
/// using the vw-css selector matching engine to determine which rules apply
/// to each element.
pub struct AuthorStyleMatcher<'a> {
    /// The parsed author stylesheet
    stylesheet: &'a Stylesheet,
    /// The DOM document (needed for selector matching)
    dom: &'a DomDocument,
    /// Mapping from NodeInfo IDs to DOM NodeIds
    node_mapping: &'a [(DomNodeId, usize)],
}

impl<'a> AuthorStyleMatcher<'a> {
    /// Create a new author style matcher.
    ///
    /// # Arguments
    ///
    /// * `stylesheet` - The parsed author stylesheet
    /// * `dom` - The DOM document
    /// * `node_mapping` - Mapping from DOM node IDs to layout node IDs
    pub fn new(
        stylesheet: &'a Stylesheet,
        dom: &'a DomDocument,
        node_mapping: &'a [(DomNodeId, usize)],
    ) -> Self {
        AuthorStyleMatcher {
            stylesheet,
            dom,
            node_mapping,
        }
    }

    /// Find the DOM node ID for a given NodeInfo ID.
    fn find_dom_node_id(&self, node_info_id: usize) -> Option<DomNodeId> {
        self.node_mapping
            .iter()
            .find(|(_, layout_id)| *layout_id == node_info_id)
            .map(|(dom_id, _)| *dom_id)
    }
}

impl<'a> RuleMatcher for AuthorStyleMatcher<'a> {
    fn match_rules(&self, node: &NodeInfo) -> Vec<MatchedRule> {
        let mut matched_rules = Vec::new();

        // Start with User-Agent defaults
        if let Some(tag) = &node.tag_name {
            let ua_rules = vw_style::defaults::user_agent_rules_for_element(tag);
            matched_rules.extend(ua_rules);
        }

        // Find the DOM node ID for this NodeInfo
        let dom_node_id = match self.find_dom_node_id(node.id) {
            Some(id) => id,
            None => return matched_rules, // Return UA rules only
        };

        // Check author stylesheet rules
        for (rule_index, rule) in self.stylesheet.rules.iter().enumerate() {
            // Check each selector in the rule
            for selector in &rule.selectors {
                if matches(selector, dom_node_id, self.dom) {
                    // Convert CSS declarations to style declarations
                    let declarations: Vec<Declaration> = rule
                        .declarations
                        .iter()
                        .filter_map(|css_decl| {
                            // Convert property name to PropertyId
                            let property = property_from_string(&css_decl.property)?;

                            // Convert CSS value to style CssValue
                            let value = convert_css_value(&css_decl.value)?;

                            Some(Declaration {
                                property,
                                value,
                                important: css_decl.important,
                            })
                        })
                        .collect();

                    if !declarations.is_empty() {
                        // Calculate specificity
                        let spec = specificity::calculate(selector);

                        matched_rules.push(MatchedRule {
                            declarations,
                            specificity: vw_style::cascade::Specificity::new(
                                spec.ids,
                                spec.classes,
                                spec.elements,
                            ),
                            origin: Origin::Author,
                            source_order: rule_index,
                        });
                    }

                    // Only need one selector to match per rule
                    break;
                }
            }
        }

        matched_rules
    }
}

/// Convert a property name string to PropertyId.
fn property_from_string(name: &str) -> Option<PropertyId> {
    match name {
        // Box model
        "display" => Some(PropertyId::Display),
        "position" => Some(PropertyId::Position),
        "width" => Some(PropertyId::Width),
        "height" => Some(PropertyId::Height),
        "min-width" => Some(PropertyId::MinWidth),
        "min-height" => Some(PropertyId::MinHeight),
        "max-width" => Some(PropertyId::MaxWidth),
        "max-height" => Some(PropertyId::MaxHeight),
        "margin-top" => Some(PropertyId::MarginTop),
        "margin-right" => Some(PropertyId::MarginRight),
        "margin-bottom" => Some(PropertyId::MarginBottom),
        "margin-left" => Some(PropertyId::MarginLeft),
        "padding-top" => Some(PropertyId::PaddingTop),
        "padding-right" => Some(PropertyId::PaddingRight),
        "padding-bottom" => Some(PropertyId::PaddingBottom),
        "padding-left" => Some(PropertyId::PaddingLeft),
        "border-top-width" => Some(PropertyId::BorderTopWidth),
        "border-right-width" => Some(PropertyId::BorderRightWidth),
        "border-bottom-width" => Some(PropertyId::BorderBottomWidth),
        "border-left-width" => Some(PropertyId::BorderLeftWidth),
        "box-sizing" => Some(PropertyId::BoxSizing),

        // Colors
        "color" => Some(PropertyId::Color),
        "background-color" => Some(PropertyId::BackgroundColor),
        "border-color" => Some(PropertyId::BorderColor),

        // Typography
        "font-size" => Some(PropertyId::FontSize),
        "font-family" => Some(PropertyId::FontFamily),
        "font-weight" => Some(PropertyId::FontWeight),
        "line-height" => Some(PropertyId::LineHeight),
        "text-align" => Some(PropertyId::TextAlign),
        "white-space" => Some(PropertyId::WhiteSpace),

        // Positioning
        "top" => Some(PropertyId::Top),
        "right" => Some(PropertyId::Right),
        "bottom" => Some(PropertyId::Bottom),
        "left" => Some(PropertyId::Left),
        "z-index" => Some(PropertyId::ZIndex),

        // Visual
        "overflow-x" => Some(PropertyId::OverflowX),
        "overflow-y" => Some(PropertyId::OverflowY),
        "visibility" => Some(PropertyId::Visibility),
        "opacity" => Some(PropertyId::Opacity),

        _ => None, // Unknown property
    }
}

/// Convert a vw-css CssValue to a vw-style CssValue.
fn convert_css_value(css_value: &vw_css::CssValue) -> Option<CssValue> {
    match css_value {
        vw_css::CssValue::Length(len) => {
            let style_length = match len.unit {
                vw_css::LengthUnit::Px => vw_style::Length::Px(len.value as f32),
                vw_css::LengthUnit::Em => vw_style::Length::Em(len.value as f32),
                vw_css::LengthUnit::Rem => vw_style::Length::Rem(len.value as f32),
                vw_css::LengthUnit::Percent => vw_style::Length::Percent(len.value as f32),
                // Unsupported units - convert to pixels using standard conversion factors
                vw_css::LengthUnit::Vw | vw_css::LengthUnit::Vh => {
                    // Skip viewport units for now (require viewport context)
                    return None;
                }
                vw_css::LengthUnit::Pt => vw_style::Length::Px((len.value * 1.333) as f32), // 1pt = 1.333px
                vw_css::LengthUnit::Cm => vw_style::Length::Px((len.value * 37.795) as f32), // 1cm = 37.795px
                vw_css::LengthUnit::Mm => vw_style::Length::Px((len.value * 3.7795) as f32), // 1mm = 3.7795px
                vw_css::LengthUnit::In => vw_style::Length::Px((len.value * 96.0) as f32),   // 1in = 96px
            };
            Some(CssValue::Length(style_length))
        }
        vw_css::CssValue::Percentage(pct) => {
            // Percentage is a Length::Percent in vw-style
            Some(CssValue::Length(vw_style::Length::Percent(*pct as f32)))
        }
        vw_css::CssValue::Color(color) => {
            // Convert alpha from 0.0-1.0 (f64) to 0-255 (u8)
            let alpha = (color.a * 255.0).round().clamp(0.0, 255.0) as u8;
            Some(CssValue::Color(Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: alpha,
            }))
        }
        vw_css::CssValue::Keyword(kw) => Some(CssValue::Keyword(kw.clone())),
        vw_css::CssValue::String(s) => Some(CssValue::String(s.clone())),
        vw_css::CssValue::Number(n) => Some(CssValue::Number(*n as f32)),
        _ => None, // Unsupported CSS value type
    }
}
