// src/unicode.rs
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// Counts the number of Unicode grapheme clusters (user-perceived characters) in a string
///
/// # Arguments
///
/// * `text` - The text to count characters in
///
/// # Returns
///
/// The number of grapheme clusters in the text
///
/// # Examples
///
/// ```
/// use dom_content_extraction::unicode::count_graphemes;
///
/// let text = "Hello, 世界!";
/// assert_eq!(count_graphemes(text), 10); // Not 13 bytes or 11 code points
/// ```
#[inline]
pub fn count_graphemes(text: &str) -> u32 {
    UnicodeSegmentation::graphemes(text, true).count() as u32
}

/// Counts the number of Unicode code points in a string
///
/// # Arguments
///
/// * `text` - The text to count code points in
///
/// # Returns
///
/// The number of Unicode code points in the text
///
/// # Examples
///
/// ```
/// use dom_content_extraction::unicode::count_code_points;
///
/// let text = "café"; // 4 letters, 5 bytes in UTF-8
/// assert_eq!(count_code_points(text), 4); // Not 5 bytes
/// ```
#[inline]
pub fn count_code_points(text: &str) -> u32 {
    text.chars().count() as u32
}

/// Normalizes text using Unicode NFC normalization
/// and trims excess whitespace
///
/// # Arguments
///
/// * `text` - The text to normalize
///
/// # Returns
///
/// A normalized string
///
/// # Examples
///
/// ```
/// use dom_content_extraction::unicode::normalize_text;
///
/// let text = "  café   \n  résumé  ";
/// assert_eq!(normalize_text(text), "café résumé");
/// ```
pub fn normalize_text(text: &str) -> String {
    // Perform NFC normalization
    let normalized = text.nfc().collect::<String>();

    // Normalize whitespace
    normalized
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Joins multiple text fragments with proper whitespace handling
///
/// # Arguments
///
/// * `fragments` - Vector of text fragments to join
///
/// # Returns
///
/// A single string with normalized whitespace between fragments
///
/// # Examples
///
/// ```
/// use dom_content_extraction::unicode::join_text_fragments;
///
/// let fragments = vec!["Hello".to_string(), "世界".to_string(), "!".to_string()];
/// assert_eq!(join_text_fragments(fragments), "Hello 世界 !");
/// ```
pub fn join_text_fragments(fragments: Vec<String>) -> String {
    let joined = fragments.join(" ");
    normalize_text(&joined)
}

/// Detects the probable script of text based on the most common script
///
/// This is a simple heuristic approach that can be useful for adjusting
/// text density calculations based on script properties
///
/// # Arguments
///
/// * `text` - The text to analyze
///
/// # Returns
///
/// A string representing the most common script in the text
///
/// # Examples
///
/// ```
/// use dom_content_extraction::unicode::detect_primary_script;
///
/// assert_eq!(detect_primary_script("Hello world"), "Latin");
/// assert_eq!(detect_primary_script("こんにちは世界"), "Han");
/// ```
pub fn detect_primary_script(text: &str) -> &'static str {
    // This is a simplified implementation
    // A production version would use the unicode_script crate
    // or implement the full Unicode Script detection algorithm

    let latin_chars = text
        .chars()
        .filter(|c| c.is_ascii() || matches!(c, 'À'..='ÿ'))
        .count();
    let cjk_chars = text
        .chars()
        .filter(|c| matches!(c, '\u{3000}'..='\u{9FFF}'))
        .count();
    let cyrillic_chars = text
        .chars()
        .filter(|c| matches!(c, '\u{0400}'..='\u{04FF}'))
        .count();

    if cjk_chars > latin_chars && cjk_chars > cyrillic_chars {
        "Han"
    } else if cyrillic_chars > latin_chars && cyrillic_chars > cjk_chars {
        "Cyrillic"
    } else {
        "Latin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_graphemes() {
        assert_eq!(count_graphemes("hello"), 5);
        assert_eq!(count_graphemes("café"), 4);
        assert_eq!(count_graphemes("こんにちは"), 5);
        // Grapheme with multiple code points (emoji with skin tone modifier)
        assert_eq!(count_graphemes("👩‍💻"), 1);
    }

    #[test]
    fn test_count_code_points() {
        assert_eq!(count_code_points("hello"), 5);
        assert_eq!(count_code_points("café"), 4);
        assert_eq!(count_code_points("こんにちは"), 5);
        // Multiple code points for a single grapheme
        assert_eq!(count_code_points("\u{1F469}\u{200D}\u{1F4BB}"), 3); // Woman technologist emoji
    }

    #[test]
    fn test_normalize_text() {
        // Combining characters
        assert_eq!(normalize_text("café"), "café");
        // Different representations of same character
        let nfd = "cafe\u{0301}"; // café with combining acute
        assert_eq!(normalize_text(nfd), "café");
        // Whitespace normalization
        assert_eq!(normalize_text("  hello  world  "), "hello world");
        assert_eq!(normalize_text("hello\n\t world"), "hello world");
    }

    #[test]
    fn test_join_text_fragments() {
        let fragments =
            vec!["Hello".to_string(), "world".to_string(), "!".to_string()];
        assert_eq!(join_text_fragments(fragments), "Hello world !");

        let fragments = vec![
            "  Text  ".to_string(),
            " with ".to_string(),
            "  extra  ".to_string(),
            " spaces ".to_string(),
        ];
        assert_eq!(join_text_fragments(fragments), "Text with extra spaces");
    }

    #[test]
    fn test_detect_primary_script() {
        assert_eq!(detect_primary_script("Hello world"), "Latin");
        assert_eq!(detect_primary_script("Привет мир"), "Cyrillic");
        assert_eq!(detect_primary_script("こんにちは世界"), "Han");
        // Mixed script with Latin dominant
        assert_eq!(detect_primary_script("Hello 世界 and more Latin"), "Latin");
    }
}
