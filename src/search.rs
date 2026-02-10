/// Fuzzy search module for tmx using nucleo-matcher.

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::types::Session;

/// Result of a fuzzy match: the session index, score, and matched char indices.
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub session_index: usize,
    pub score: u32,
    pub indices: Vec<u32>,
}

/// Perform fuzzy matching of `query` against a list of sessions.
/// Returns matched sessions sorted by score (highest first).
/// Empty query returns all sessions with score 0.
pub fn fuzzy_match_sessions(sessions: &[Session], query: &str) -> Vec<MatchResult> {
    if query.is_empty() {
        return sessions
            .iter()
            .enumerate()
            .map(|(i, _)| MatchResult {
                session_index: i,
                score: 0,
                indices: Vec::new(),
            })
            .collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::new(query, CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy);

    let mut results: Vec<MatchResult> = Vec::new();
    let mut buf = Vec::new();

    for (i, session) in sessions.iter().enumerate() {
        let haystack = Utf32Str::new(&session.name, &mut buf);
        let mut indices = Vec::new();
        if let Some(score) = pattern.indices(haystack, &mut matcher, &mut indices) {
            indices.sort_unstable();
            indices.dedup();
            results.push(MatchResult {
                session_index: i,
                score,
                indices,
            });
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Session;
    use std::time::Instant;

    fn make_session(name: &str) -> Session {
        Session {
            id: format!("${name}"),
            name: name.to_string(),
            windows: 1,
            attached: 0,
            created: 0,
            last_attached: 0,
            group: None,
            path: "/tmp".to_string(),
        }
    }

    #[test]
    fn test_fuzzy_exact_match() {
        let sessions = vec![
            make_session("work"),
            make_session("personal"),
            make_session("dev"),
        ];

        let results = fuzzy_match_sessions(&sessions, "work");
        assert!(!results.is_empty(), "exact match should return results");
        assert_eq!(
            sessions[results[0].session_index].name, "work",
            "first result should be 'work'"
        );
        assert!(results[0].score > 0, "exact match should have positive score");
    }

    #[test]
    fn test_fuzzy_partial_match() {
        let sessions = vec![
            make_session("work"),
            make_session("personal"),
            make_session("dev"),
        ];

        let results = fuzzy_match_sessions(&sessions, "wrk");
        assert!(!results.is_empty(), "partial match 'wrk' should match 'work'");
        assert!(
            results
                .iter()
                .any(|r| sessions[r.session_index].name == "work"),
            "'work' should be in results for 'wrk'"
        );
    }

    #[test]
    fn test_fuzzy_empty_query() {
        let sessions = vec![
            make_session("alpha"),
            make_session("beta"),
            make_session("gamma"),
        ];

        let results = fuzzy_match_sessions(&sessions, "");
        assert_eq!(
            results.len(),
            3,
            "empty query should return all sessions"
        );
        for r in &results {
            assert_eq!(r.score, 0, "empty query score should be 0");
        }
    }

    #[test]
    fn test_fuzzy_no_match() {
        let sessions = vec![
            make_session("work"),
            make_session("personal"),
            make_session("dev"),
        ];

        let results = fuzzy_match_sessions(&sessions, "xyz123");
        assert!(
            results.is_empty(),
            "query 'xyz123' should match nothing, got {} results",
            results.len()
        );
    }

    #[test]
    fn test_fuzzy_special_chars() {
        let sessions = vec![
            make_session("데모세션"),
            make_session("work"),
            make_session("개발서버"),
        ];

        let results = fuzzy_match_sessions(&sessions, "데모");
        assert!(
            !results.is_empty(),
            "Korean query '데모' should match '데모세션'"
        );
        assert_eq!(
            sessions[results[0].session_index].name, "데모세션",
            "first result should be '데모세션'"
        );
    }

    #[test]
    fn test_fuzzy_performance() {
        let sessions: Vec<Session> = (0..100)
            .map(|i| make_session(&format!("session-{i:04}-{}", "x".repeat(20))))
            .collect();

        let start = Instant::now();
        let _results = fuzzy_match_sessions(&sessions, "sess42");
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 50,
            "100 sessions fuzzy match should complete in <50ms, took {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_fuzzy_match_indices_returned() {
        let sessions = vec![make_session("work")];
        let results = fuzzy_match_sessions(&sessions, "wk");
        assert!(!results.is_empty());
        let indices = &results[0].indices;
        assert!(
            !indices.is_empty(),
            "match indices should be non-empty for a match"
        );
        assert!(
            indices.contains(&0),
            "index 0 ('w') should be in matched indices"
        );
        assert!(
            indices.contains(&3),
            "index 3 ('k') should be in matched indices"
        );
    }

    #[test]
    fn test_fuzzy_case_insensitive() {
        let sessions = vec![make_session("WorkStation"), make_session("dev")];
        let results = fuzzy_match_sessions(&sessions, "work");
        assert!(
            !results.is_empty(),
            "case-insensitive match: 'work' should match 'WorkStation'"
        );
        assert_eq!(sessions[results[0].session_index].name, "WorkStation");
    }
}
