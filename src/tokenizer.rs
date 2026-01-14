//! Token extraction from grid strings

/// A warning generated during tokenization
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub message: String,
}

impl Warning {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Extracts tokens from a grid row string.
///
/// Tokens are of the form `{name}` where name can contain any characters
/// except `}`. Characters outside of tokens generate warnings.
///
/// # Examples
///
/// ```
/// use pxl::tokenizer::tokenize;
///
/// let (tokens, warnings) = tokenize("{a}{b}{c}");
/// assert_eq!(tokens, vec!["{a}", "{b}", "{c}"]);
/// assert!(warnings.is_empty());
///
/// let (tokens, warnings) = tokenize("x{a}y");
/// assert_eq!(tokens, vec!["{a}"]);
/// assert_eq!(warnings.len(), 2); // warnings for 'x' and 'y'
/// ```
pub fn tokenize(row: &str) -> (Vec<String>, Vec<Warning>) {
    let mut tokens = Vec::new();
    let mut warnings = Vec::new();
    let mut chars = row.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            // Start of a token
            let mut token = String::from("{");
            let mut closed = false;

            for inner in chars.by_ref() {
                token.push(inner);
                if inner == '}' {
                    closed = true;
                    break;
                }
            }

            if closed {
                tokens.push(token);
            } else {
                // Unclosed token
                warnings.push(Warning::new(format!(
                    "Unclosed token '{}' in grid row",
                    token
                )));
            }
        } else {
            // Character outside token
            warnings.push(Warning::new(format!(
                "Unexpected character '{}' in grid row",
                c
            )));
        }
    }

    (tokens, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let (tokens, warnings) = tokenize("{a}{b}{c}");
        assert_eq!(tokens, vec!["{a}", "{b}", "{c}"]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_extra_characters() {
        let (tokens, warnings) = tokenize("x{a}y");
        assert_eq!(tokens, vec!["{a}"]);
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].message.contains("'x'"));
        assert!(warnings[1].message.contains("'y'"));
    }

    #[test]
    fn test_unclosed_token() {
        let (tokens, warnings) = tokenize("{unclosed");
        assert!(tokens.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("Unclosed"));
    }

    #[test]
    fn test_empty_string() {
        let (tokens, warnings) = tokenize("");
        assert!(tokens.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_longer_token_names() {
        let (tokens, warnings) = tokenize("{_}{skin}{_}");
        assert_eq!(tokens, vec!["{_}", "{skin}", "{_}"]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_complex_token_names() {
        let (tokens, warnings) = tokenize("{long_name}{x}");
        assert_eq!(tokens, vec!["{long_name}", "{x}"]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_multiple_extra_chars() {
        let (tokens, warnings) = tokenize("abc{x}def{x}ghi");
        assert_eq!(tokens, vec!["{x}", "{x}"]);
        assert_eq!(warnings.len(), 9); // a,b,c,d,e,f,g,h,i
    }
}
