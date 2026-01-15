//! Typo suggestions using Levenshtein distance

/// Calculate the Levenshtein distance between two strings.
/// This measures the minimum number of single-character edits (insertions,
/// deletions, or substitutions) required to change one string into the other.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Handle empty strings
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Use two rows instead of full matrix for O(min(m,n)) space
    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row: Vec<usize> = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr_row[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr_row[j] = (prev_row[j] + 1) // deletion
                .min(curr_row[j - 1] + 1) // insertion
                .min(prev_row[j - 1] + cost); // substitution
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// Find suggestions for a typo from a list of valid options.
/// Returns suggestions sorted by edit distance (closest first).
///
/// - `query`: The mistyped string
/// - `candidates`: List of valid options to compare against
/// - `max_distance`: Maximum edit distance to consider (default: 3)
///
/// Returns up to 3 closest matches within the max distance.
pub fn suggest<'a>(query: &str, candidates: &[&'a str], max_distance: usize) -> Vec<&'a str> {
    let query_lower = query.to_lowercase();

    let mut scored: Vec<(&str, usize)> = candidates
        .iter()
        .map(|&candidate| {
            let candidate_lower = candidate.to_lowercase();
            let distance = levenshtein_distance(&query_lower, &candidate_lower);
            (candidate, distance)
        })
        .filter(|(_, distance)| *distance <= max_distance)
        .collect();

    // Sort by distance (ascending)
    scored.sort_by_key(|(_, distance)| *distance);

    // Return up to 3 closest matches
    scored.into_iter().take(3).map(|(s, _)| s).collect()
}

/// Format a "did you mean?" suggestion string.
/// Returns None if there are no suggestions.
pub fn format_suggestion(suggestions: &[&str]) -> Option<String> {
    match suggestions.len() {
        0 => None,
        1 => Some(format!("Did you mean '{}'?", suggestions[0])),
        2 => Some(format!(
            "Did you mean '{}' or '{}'?",
            suggestions[0], suggestions[1]
        )),
        _ => Some(format!(
            "Did you mean '{}', '{}', or '{}'?",
            suggestions[0], suggestions[1], suggestions[2]
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", "hello"), 5);
        assert_eq!(levenshtein_distance("hello", ""), 5);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_single_edit() {
        // Substitution
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
        // Insertion
        assert_eq!(levenshtein_distance("hello", "helllo"), 1);
        // Deletion
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
    }

    #[test]
    fn test_levenshtein_multiple_edits() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_suggest_exact_match() {
        let candidates = &["character", "item", "tileset", "animation"];
        let suggestions = suggest("character", candidates, 3);
        assert_eq!(suggestions, vec!["character"]);
    }

    #[test]
    fn test_suggest_typo() {
        let candidates = &["character", "item", "tileset", "animation"];
        // "charactor" is 1 edit from "character"
        let suggestions = suggest("charactor", candidates, 3);
        assert_eq!(suggestions[0], "character");
    }

    #[test]
    fn test_suggest_case_insensitive() {
        let candidates = &["character", "Item", "TILESET"];
        let suggestions = suggest("CHARACTER", candidates, 3);
        assert_eq!(suggestions[0], "character");
    }

    #[test]
    fn test_suggest_no_match() {
        let candidates = &["character", "item", "tileset"];
        let suggestions = suggest("xyz", candidates, 2);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_format_suggestion_none() {
        assert_eq!(format_suggestion(&[]), None);
    }

    #[test]
    fn test_format_suggestion_one() {
        let result = format_suggestion(&["character"]);
        assert_eq!(result, Some("Did you mean 'character'?".to_string()));
    }

    #[test]
    fn test_format_suggestion_two() {
        let result = format_suggestion(&["character", "item"]);
        assert_eq!(
            result,
            Some("Did you mean 'character' or 'item'?".to_string())
        );
    }

    #[test]
    fn test_format_suggestion_three() {
        let result = format_suggestion(&["character", "item", "tileset"]);
        assert_eq!(
            result,
            Some("Did you mean 'character', 'item', or 'tileset'?".to_string())
        );
    }
}
