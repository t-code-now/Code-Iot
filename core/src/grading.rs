//! Checking answers.

use crate::api::Answer;
use crate::model::Step;

/// The result of checking one answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub correct: bool,
    pub expected: String,
    pub explanation: &'static str,
}

/// Canonical form for typed answers: whitespace removed, single quotes
/// treated as double quotes. `print( 'hi' )` == `print("hi")`.
pub fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == '\'' { '"' } else { c })
        .collect()
}

/// Check `answer` against `step`. Returns `None` for `Learn` steps, which
/// are not graded. An answer of the wrong kind is simply incorrect.
pub fn check(step: &Step, answer: &Answer) -> Option<Verdict> {
    let verdict = match step {
        Step::Learn { .. } => return None,
        Step::MultipleChoice { options, correct, explanation, .. } => Verdict {
            correct: matches!(answer, Answer::Choice(i) if i == correct),
            expected: options.get(*correct).copied().unwrap_or_default().to_string(),
            explanation,
        },
        Step::FillBlanks { answers, explanation, .. } => Verdict {
            correct: matches!(answer, Answer::Tokens(t) if tokens_eq(t, answers)),
            expected: answers.join("  ·  "),
            explanation,
        },
        Step::Reorder { lines, explanation, .. } => Verdict {
            correct: matches!(answer, Answer::Tokens(t) if tokens_eq(t, lines)),
            expected: lines.join("\n"),
            explanation,
        },
        Step::TypeIn { accepted, explanation, .. } => Verdict {
            correct: match answer {
                Answer::Text(t) => {
                    let n = normalize(t);
                    !n.is_empty() && accepted.iter().any(|a| normalize(a) == n)
                }
                _ => false,
            },
            expected: accepted.first().copied().unwrap_or_default().to_string(),
            explanation,
        },
    };
    Some(verdict)
}

fn tokens_eq(given: &[String], expected: &[&str]) -> bool {
    given.len() == expected.len() && given.iter().zip(expected).all(|(g, e)| g.trim() == e.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_choice() {
        let s = Step::MultipleChoice {
            prompt: "?",
            code: None,
            options: vec!["a", "b"],
            correct: 1,
            explanation: "",
        };
        assert!(check(&s, &Answer::Choice(1)).unwrap().correct);
        assert!(!check(&s, &Answer::Choice(0)).unwrap().correct);
        assert!(!check(&s, &Answer::Text("b".into())).unwrap().correct);
        assert_eq!(check(&s, &Answer::Choice(0)).unwrap().expected, "b");
    }

    #[test]
    fn fill_and_reorder() {
        let f = Step::FillBlanks {
            prompt: "",
            code: "___ = ___",
            options: vec!["x", "5", "y"],
            answers: vec!["x", "5"],
            explanation: "",
        };
        assert!(check(&f, &Answer::Tokens(vec!["x".into(), "5".into()])).unwrap().correct);
        assert!(!check(&f, &Answer::Tokens(vec!["5".into(), "x".into()])).unwrap().correct);
        assert!(!check(&f, &Answer::Tokens(vec!["x".into()])).unwrap().correct);

        let r = Step::Reorder { prompt: "", lines: vec!["a", "b", "c"], explanation: "" };
        let ok: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        assert!(check(&r, &Answer::Tokens(ok)).unwrap().correct);
    }

    #[test]
    fn type_in_is_lenient_about_spacing_and_quotes() {
        let s = Step::TypeIn {
            prompt: "",
            code: None,
            accepted: vec!["print(\"hi\")"],
            explanation: "",
        };
        assert!(check(&s, &Answer::Text("print( 'hi' )".into())).unwrap().correct);
        assert!(!check(&s, &Answer::Text("Print('hi')".into())).unwrap().correct);
        assert!(!check(&s, &Answer::Text("   ".into())).unwrap().correct);
    }

    #[test]
    fn learn_is_not_graded() {
        let s = Step::Learn { title: "", body: "", code: None };
        assert!(check(&s, &Answer::Choice(0)).is_none());
    }
}
