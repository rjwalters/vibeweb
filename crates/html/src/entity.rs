//! HTML entity decoding
//!
//! Provides functionality to decode HTML entities in text and attribute values.
//! Supports both named entities (&amp;, &lt;, etc.) and numeric entities
//! (&#60;, &#x3C;).

/// Decodes HTML entities in a string.
///
/// Processes both named entities (e.g., `&amp;`, `&lt;`) and numeric entities
/// (e.g., `&#60;`, &#x3C;`). Invalid or unknown entities are passed through
/// unchanged.
///
/// # Examples
///
/// ```
/// use vw_html::decode_entities;
///
/// assert_eq!(decode_entities("Hello &amp; goodbye"), "Hello & goodbye");
/// assert_eq!(decode_entities("&#60;script&#62;"), "<script>");
/// assert_eq!(decode_entities("&unknown;"), "&unknown;");
/// ```
pub fn decode_entities(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' {
            // Collect the potential entity
            let mut entity_str = String::from("&");
            let mut entity_chars = Vec::new();

            while let Some(&next_char) = chars.peek() {
                if next_char == ';' {
                    entity_chars.push(chars.next().unwrap());
                    entity_str.push(next_char);
                    break;
                }
                if entity_chars.len() > 20
                    || !next_char.is_ascii_alphanumeric()
                        && next_char != '#'
                        && next_char != 'x'
                        && next_char != 'X'
                {
                    // Too long or invalid character - not an entity
                    break;
                }
                entity_chars.push(chars.next().unwrap());
                entity_str.push(next_char);
            }

            // Try to decode the entity
            if let Some(decoded) = try_decode_entity(&entity_str) {
                result.push_str(&decoded);
            } else {
                // Not a valid entity, keep the original string
                result.push_str(&entity_str);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Attempts to decode a complete entity string (including & and optional ;).
///
/// Returns `Some(String)` if a valid entity was decoded, `None` otherwise.
fn try_decode_entity(entity_str: &str) -> Option<String> {
    if !entity_str.starts_with('&') {
        return None;
    }

    let inner = &entity_str[1..];

    // Entities must end with semicolon
    if !inner.ends_with(';') {
        return None;
    }

    // Check for numeric entity
    if let Some(stripped) = inner.strip_prefix('#') {
        try_decode_numeric_entity(stripped)
    } else {
        // Named entity - remove the trailing semicolon
        let name = &inner[..inner.len() - 1];
        NAMED_ENTITIES.get(name).map(|&s| s.to_string())
    }
}

/// Attempts to decode a numeric entity from the content after the '#'.
///
/// Handles: 123 (decimal), x7B or X7B (hexadecimal, case-insensitive)
/// Input should NOT include the leading # or trailing ;
fn try_decode_numeric_entity(content: &str) -> Option<String> {
    let content = content.trim_end_matches(';');

    if content.is_empty() {
        return None;
    }

    let (is_hex, digits) = if content.starts_with('x') || content.starts_with('X') {
        (true, &content[1..])
    } else {
        (false, content)
    };

    if digits.is_empty() {
        return None;
    }

    // Parse the number
    let code = if is_hex {
        u32::from_str_radix(digits, 16).ok()?
    } else {
        digits.parse::<u32>().ok()?
    };

    // Convert to character
    char::from_u32(code).map(|c| c.to_string())
}

/// Common HTML5 named entities.
///
/// This table includes the most frequently used named entities. For a complete
/// list, see: https://html.spec.whatwg.org/multipage/named-characters.html
static NAMED_ENTITIES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // Basic entities
    "amp" => "&",
    "lt" => "<",
    "gt" => ">",
    "quot" => "\"",
    "apos" => "'",

    // Whitespace
    "nbsp" => "\u{00A0}",  // non-breaking space
    "ensp" => "\u{2002}",  // en space
    "emsp" => "\u{2003}",  // em space
    "thinsp" => "\u{2009}", // thin space

    // Punctuation
    "ndash" => "–",
    "mdash" => "—",
    "lsquo" => "'",
    "rsquo" => "'",
    "ldquo" => "\u{201C}",
    "rdquo" => "\u{201D}",
    "sbquo" => "‚",
    "bdquo" => "„",
    "hellip" => "…",
    "prime" => "′",
    "Prime" => "″",

    // Currency
    "cent" => "¢",
    "pound" => "£",
    "yen" => "¥",
    "euro" => "€",
    "curren" => "¤",

    // Math & Technical
    "times" => "×",
    "divide" => "÷",
    "minus" => "−",
    "plusmn" => "±",
    "frac14" => "¼",
    "frac12" => "½",
    "frac34" => "¾",
    "sup1" => "¹",
    "sup2" => "²",
    "sup3" => "³",
    "deg" => "°",
    "micro" => "µ",
    "middot" => "·",

    // Symbols
    "copy" => "©",
    "reg" => "®",
    "trade" => "™",
    "sect" => "§",
    "para" => "¶",
    "dagger" => "†",
    "Dagger" => "‡",
    "bull" => "•",

    // Arrows
    "larr" => "←",
    "uarr" => "↑",
    "rarr" => "→",
    "darr" => "↓",
    "harr" => "↔",
    "lArr" => "⇐",
    "uArr" => "⇑",
    "rArr" => "⇒",
    "dArr" => "⇓",
    "hArr" => "⇔",

    // Greek letters (lowercase)
    "alpha" => "α",
    "beta" => "β",
    "gamma" => "γ",
    "delta" => "δ",
    "epsilon" => "ε",
    "zeta" => "ζ",
    "eta" => "η",
    "theta" => "θ",
    "iota" => "ι",
    "kappa" => "κ",
    "lambda" => "λ",
    "mu" => "μ",
    "nu" => "ν",
    "xi" => "ξ",
    "omicron" => "ο",
    "pi" => "π",
    "rho" => "ρ",
    "sigma" => "σ",
    "tau" => "τ",
    "upsilon" => "υ",
    "phi" => "φ",
    "chi" => "χ",
    "psi" => "ψ",
    "omega" => "ω",

    // Greek letters (uppercase)
    "Alpha" => "Α",
    "Beta" => "Β",
    "Gamma" => "Γ",
    "Delta" => "Δ",
    "Epsilon" => "Ε",
    "Zeta" => "Ζ",
    "Eta" => "Η",
    "Theta" => "Θ",
    "Iota" => "Ι",
    "Kappa" => "Κ",
    "Lambda" => "Λ",
    "Mu" => "Μ",
    "Nu" => "Ν",
    "Xi" => "Ξ",
    "Omicron" => "Ο",
    "Pi" => "Π",
    "Rho" => "Ρ",
    "Sigma" => "Σ",
    "Tau" => "Τ",
    "Upsilon" => "Υ",
    "Phi" => "Φ",
    "Chi" => "Χ",
    "Psi" => "Ψ",
    "Omega" => "Ω",

    // Latin Extended-A
    "Agrave" => "À",
    "Aacute" => "Á",
    "Acirc" => "Â",
    "Atilde" => "Ã",
    "Auml" => "Ä",
    "Aring" => "Å",
    "AElig" => "Æ",
    "Ccedil" => "Ç",
    "Egrave" => "È",
    "Eacute" => "É",
    "Ecirc" => "Ê",
    "Euml" => "Ë",
    "Igrave" => "Ì",
    "Iacute" => "Í",
    "Icirc" => "Î",
    "Iuml" => "Ï",
    "ETH" => "Ð",
    "Ntilde" => "Ñ",
    "Ograve" => "Ò",
    "Oacute" => "Ó",
    "Ocirc" => "Ô",
    "Otilde" => "Õ",
    "Ouml" => "Ö",
    "Oslash" => "Ø",
    "Ugrave" => "Ù",
    "Uacute" => "Ú",
    "Ucirc" => "Û",
    "Uuml" => "Ü",
    "Yacute" => "Ý",
    "THORN" => "Þ",
    "szlig" => "ß",
    "agrave" => "à",
    "aacute" => "á",
    "acirc" => "â",
    "atilde" => "ã",
    "auml" => "ä",
    "aring" => "å",
    "aelig" => "æ",
    "ccedil" => "ç",
    "egrave" => "è",
    "eacute" => "é",
    "ecirc" => "ê",
    "euml" => "ë",
    "igrave" => "ì",
    "iacute" => "í",
    "icirc" => "î",
    "iuml" => "ï",
    "eth" => "ð",
    "ntilde" => "ñ",
    "ograve" => "ò",
    "oacute" => "ó",
    "ocirc" => "ô",
    "otilde" => "õ",
    "ouml" => "ö",
    "oslash" => "ø",
    "ugrave" => "ù",
    "uacute" => "ú",
    "ucirc" => "û",
    "uuml" => "ü",
    "yacute" => "ý",
    "thorn" => "þ",
    "yuml" => "ÿ",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_named_entities() {
        assert_eq!(decode_entities("&amp;"), "&");
        assert_eq!(decode_entities("&lt;"), "<");
        assert_eq!(decode_entities("&gt;"), ">");
        assert_eq!(decode_entities("&quot;"), "\"");
        assert_eq!(decode_entities("&apos;"), "'");
    }

    #[test]
    fn test_nbsp() {
        assert_eq!(decode_entities("&nbsp;"), "\u{00A0}");
    }

    #[test]
    fn test_combined_text() {
        assert_eq!(decode_entities("Hello &amp; goodbye"), "Hello & goodbye");
        assert_eq!(decode_entities("&lt;div&gt;"), "<div>");
        assert_eq!(decode_entities("A &amp; B &lt; C &gt; D"), "A & B < C > D");
    }

    #[test]
    fn test_decimal_numeric_entities() {
        assert_eq!(decode_entities("&#60;"), "<");
        assert_eq!(decode_entities("&#62;"), ">");
        assert_eq!(decode_entities("&#38;"), "&");
        assert_eq!(decode_entities("&#160;"), "\u{00A0}");
    }

    #[test]
    fn test_hex_numeric_entities() {
        assert_eq!(decode_entities("&#x3C;"), "<");
        assert_eq!(decode_entities("&#x3E;"), ">");
        assert_eq!(decode_entities("&#x26;"), "&");
        assert_eq!(decode_entities("&#xA0;"), "\u{00A0}");
    }

    #[test]
    fn test_hex_case_insensitive() {
        assert_eq!(decode_entities("&#x3c;"), "<");
        assert_eq!(decode_entities("&#X3C;"), "<");
        assert_eq!(decode_entities("&#X3c;"), "<");
    }

    #[test]
    fn test_unicode_emoji() {
        // 😀 (grinning face) = U+1F600
        assert_eq!(decode_entities("&#128512;"), "😀");
        assert_eq!(decode_entities("&#x1F600;"), "😀");
    }

    #[test]
    fn test_unknown_entity_passthrough() {
        assert_eq!(decode_entities("&unknown;"), "&unknown;");
        assert_eq!(decode_entities("&notreal;"), "&notreal;");
    }

    #[test]
    fn test_unterminated_entity() {
        // Without semicolon, should pass through
        assert_eq!(decode_entities("&amp"), "&amp");
        assert_eq!(decode_entities("&#60"), "&#60");
    }

    #[test]
    fn test_invalid_numeric_entity() {
        // Invalid code points should pass through
        assert_eq!(decode_entities("&#999999999;"), "&#999999999;");
        assert_eq!(decode_entities("&#xFFFFFFFF;"), "&#xFFFFFFFF;");
    }

    #[test]
    fn test_empty_entity() {
        assert_eq!(decode_entities("&;"), "&;");
        assert_eq!(decode_entities("&#;"), "&#;");
    }

    #[test]
    fn test_special_chars() {
        assert_eq!(decode_entities("&copy;"), "©");
        assert_eq!(decode_entities("&reg;"), "®");
        assert_eq!(decode_entities("&trade;"), "™");
        assert_eq!(decode_entities("&euro;"), "€");
    }

    #[test]
    fn test_multiple_entities_in_sequence() {
        assert_eq!(decode_entities("&lt;&gt;&amp;"), "<>&");
        assert_eq!(decode_entities("&#60;&#62;&#38;"), "<>&");
    }

    #[test]
    fn test_mixed_numeric_and_named() {
        assert_eq!(decode_entities("&lt;&#62;&amp;"), "<>&");
        assert_eq!(decode_entities("&#60;&gt;&#38;"), "<>&");
    }

    #[test]
    fn test_real_world_example() {
        assert_eq!(
            decode_entities("HTML &amp; CSS &ndash; The Complete Guide"),
            "HTML & CSS – The Complete Guide"
        );
    }
}
