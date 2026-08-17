//! Fuzzy subsequence matching, used to filter commands by query text.

/// Returns a score if every character of `query` appears in order within
/// `target` (case-insensitive), and `None` otherwise.
///
/// Better matches score higher, so results can be sorted most-relevant first:
/// - matches at the start of the target
/// - matches at word boundaries (following a separator)
/// - consecutive runs of matched characters
/// - small gaps between matched characters
pub fn fuzzy_match(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let query_chars: Vec<char> = query.chars().map(lower_char).collect();
    let target_chars: Vec<char> = target.chars().collect();

    let mut query_index = 0;
    let mut score = 0;
    let mut last_match: Option<usize> = None;

    for (index, &character) in target_chars.iter().enumerate() {
        if query_index == query_chars.len() {
            break;
        }
        if lower_char(character) != query_chars[query_index] {
            continue;
        }

        let mut char_score = 2;
        if index == 0 || is_separator(target_chars[index - 1]) {
            char_score += 3;
        }
        match last_match {
            Some(previous) if previous + 1 == index => char_score += 2,
            Some(previous) => char_score -= (index - previous - 1).min(2) as i32,
            None => {}
        }
        score += char_score;
        last_match = Some(index);
        query_index += 1;
    }

    if query_index == query_chars.len() {
        Some(score)
    } else {
        None
    }
}

fn lower_char(character: char) -> char {
    character.to_lowercase().next().unwrap_or(character)
}

fn is_separator(character: char) -> bool {
    !character.is_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::fuzzy_match;

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(fuzzy_match("", "Toggle Sidebar"), Some(0));
    }

    #[test]
    fn matches_subsequence_case_insensitively() {
        assert!(fuzzy_match("tgsb", "Toggle Sidebar").is_some());
        assert!(fuzzy_match("TGSB", "Toggle Sidebar").is_some());
    }

    #[test]
    fn rejects_out_of_order_and_absent_chars() {
        assert!(fuzzy_match("bts", "Toggle Sidebar").is_none());
        assert!(fuzzy_match("zzz", "Toggle Sidebar").is_none());
    }

    #[test]
    fn prefix_scores_higher_than_scattered() {
        let prefix = fuzzy_match("toggle", "Toggle Sidebar").unwrap();
        let scattered = fuzzy_match("tgsb", "Toggle Sidebar").unwrap();
        assert!(prefix > scattered);
    }

    #[test]
    fn word_boundary_scores_higher_than_middle_of_word() {
        let word_start = fuzzy_match("sb", "Toggle Sidebar").unwrap();
        let middle = fuzzy_match("gb", "Toggle Sidebar").unwrap();
        assert!(word_start > middle);
    }

    #[test]
    fn consecutive_run_scores_higher_than_gapped() {
        let consecutive = fuzzy_match("side", "Toggle Sidebar").unwrap();
        let gapped = fuzzy_match("sbr", "Toggle Sidebar").unwrap();
        assert!(consecutive > gapped);
    }
}
