//! Translations of course content.
//!
//! English lives in the Rust content files. Other languages are plain text
//! files under `content/<lang>/`, one per course, with one `key = value`
//! line per translated string:
//!
//! ```text
//! course.python.description = …
//! section.py-s1 = …                 (section title)
//! py-1-1 = …                        (lesson title)
//! py-1-1.0.title = …                (Learn card title)
//! py-1-1.0.body = …                 (Learn card text)
//! py-1-1.1.prompt = …               (exercise question)
//! py-1-1.1.explanation = …          (shown after checking)
//! py-1-1.1.opt2 = …                 (multiple-choice option #2; only prose options)
//! ```
//!
//! Code (snippets, tiles, lines, accepted answers) is never translated.
//! Anything missing falls back to English. `\n` in a value is a line break.

use std::collections::HashMap;
use std::sync::OnceLock;

/// A UI/content language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Lang {
    #[default]
    En,
    He,
}

impl Lang {
    /// Parse a language code such as "he", "he-IL" or "en". Unknown → English.
    pub fn parse(code: &str) -> Lang {
        let code = code.trim().to_ascii_lowercase();
        if code == "he" || code.starts_with("he-") || code == "iw" {
            Lang::He
        } else {
            Lang::En
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::He => "he",
        }
    }

    pub fn is_rtl(self) -> bool {
        matches!(self, Lang::He)
    }
}

const HE_FILES: &[(&str, &str)] = &[
    ("python", include_str!("content/he/python.txt")),
    ("javascript", include_str!("content/he/javascript.txt")),
    ("web", include_str!("content/he/web.txt")),
    ("sql", include_str!("content/he/sql.txt")),
    ("rust", include_str!("content/he/rust.txt")),
    ("cpp", include_str!("content/he/cpp.txt")),
    ("arduino", include_str!("content/he/arduino.txt")),
];

/// One parsed line: (key, value, source file, line number).
pub(crate) type Entry = (&'static str, String, &'static str, usize);

fn unescape(v: &str) -> String {
    v.replace("\\n", "\n")
}

/// Parse a translation file. Blank lines and lines starting with `//` are ignored.
pub(crate) fn parse_file(name: &'static str, text: &'static str) -> Vec<Entry> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some((k, v)) = line.split_once(" = ") {
            out.push((k.trim(), unescape(v.trim()), name, i + 1));
        } else {
            // Kept so tests can report malformed lines.
            out.push(("", line.to_string(), name, i + 1));
        }
    }
    out
}

pub(crate) fn entries(lang: Lang) -> Vec<Entry> {
    match lang {
        Lang::En => Vec::new(),
        Lang::He => HE_FILES.iter().flat_map(|(n, t)| parse_file(n, t)).collect(),
    }
}

fn table(lang: Lang) -> Option<&'static HashMap<&'static str, String>> {
    static HE: OnceLock<HashMap<&'static str, String>> = OnceLock::new();
    match lang {
        Lang::En => None,
        Lang::He => Some(HE.get_or_init(|| {
            entries(Lang::He).into_iter().filter(|e| !e.0.is_empty()).map(|(k, v, _, _)| (k, v)).collect()
        })),
    }
}

/// The translation for `key`, if one exists.
pub fn lookup(lang: Lang, key: &str) -> Option<&'static str> {
    table(lang)?.get(key).map(String::as_str)
}

/// The translation for `key`, or `fallback` (the English text).
pub fn text(lang: Lang, key: &str, fallback: &'static str) -> &'static str {
    lookup(lang, key).unwrap_or(fallback)
}

pub fn course_title(lang: Lang, course_id: &str, fallback: &'static str) -> &'static str {
    text(lang, &format!("course.{course_id}.title"), fallback)
}

pub fn course_description(lang: Lang, course_id: &str, fallback: &'static str) -> &'static str {
    text(lang, &format!("course.{course_id}.description"), fallback)
}

pub fn section_title(lang: Lang, section_id: &str, fallback: &'static str) -> &'static str {
    text(lang, &format!("section.{section_id}"), fallback)
}

pub fn lesson_title(lang: Lang, lesson_id: &str, fallback: &'static str) -> &'static str {
    text(lang, lesson_id, fallback)
}

/// A field of a lesson step: "title", "body", "prompt", "explanation" or "opt<N>".
pub fn step_text(lang: Lang, lesson_id: &str, step: usize, field: &str, fallback: &'static str) -> &'static str {
    text(lang, &format!("{lesson_id}.{step}.{field}"), fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::model::Step;
    use std::collections::HashSet;

    #[test]
    fn parse_codes() {
        assert_eq!(Lang::parse("he"), Lang::He);
        assert_eq!(Lang::parse("he-IL"), Lang::He);
        assert_eq!(Lang::parse("en-US"), Lang::En);
        assert_eq!(Lang::parse(""), Lang::En);
        assert!(Lang::He.is_rtl() && !Lang::En.is_rtl());
    }

    #[test]
    fn fallback_is_english() {
        assert_eq!(text(Lang::En, "py-1-1", "Your first program"), "Your first program");
        assert_eq!(text(Lang::He, "no.such.key", "x"), "x");
    }

    /// Every key must point at a real field, and appear only once.
    #[test]
    fn hebrew_keys_are_valid() {
        let mut problems = Vec::new();
        let mut seen = HashSet::new();
        let lessons: Vec<_> = catalog().iter().flat_map(|c| c.lessons()).collect();
        let sections: HashSet<&str> = catalog().iter().flat_map(|c| c.sections.iter().map(|s| s.id)).collect();
        let courses: HashSet<&str> = catalog().iter().map(|c| c.id).collect();

        for (key, value, file, line) in entries(Lang::He) {
            let at = format!("{file}.txt:{line}");
            if key.is_empty() {
                problems.push(format!("{at}: missing ' = '"));
                continue;
            }
            if value.is_empty() {
                problems.push(format!("{at}: empty value for {key}"));
            }
            if !seen.insert(key) {
                problems.push(format!("{at}: duplicate key {key}"));
            }
            let parts: Vec<&str> = key.split('.').collect();
            let ok = match parts.as_slice() {
                ["course", id, "title" | "description"] => courses.contains(id),
                ["section", id] => sections.contains(id),
                [lesson_id] => lessons.iter().any(|l| l.id == *lesson_id),
                [lesson_id, idx, field] => {
                    match (lessons.iter().find(|l| l.id == *lesson_id), idx.parse::<usize>()) {
                        (Some(l), Ok(i)) => match l.steps.get(i) {
                            Some(Step::Learn { .. }) => matches!(*field, "title" | "body"),
                            Some(Step::MultipleChoice { options, .. }) => {
                                matches!(*field, "prompt" | "explanation")
                                    || field
                                        .strip_prefix("opt")
                                        .and_then(|n| n.parse::<usize>().ok())
                                        .is_some_and(|n| n < options.len())
                            }
                            Some(_) => matches!(*field, "prompt" | "explanation"),
                            None => false,
                        },
                        _ => false,
                    }
                }
                _ => false,
            };
            if !ok {
                problems.push(format!("{at}: unknown key {key}"));
            }
        }
        assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
    }

    /// Every prose field in every course must have a Hebrew translation.
    #[test]
    fn hebrew_covers_all_prose() {
        let mut missing = Vec::new();
        let mut need = |key: String| {
            if lookup(Lang::He, &key).is_none() {
                missing.push(key);
            }
        };
        for c in catalog() {
            need(format!("course.{}.description", c.id));
            for s in &c.sections {
                need(format!("section.{}", s.id));
                for l in &s.lessons {
                    need(l.id.to_string());
                    for (i, step) in l.steps.iter().enumerate() {
                        match step {
                            Step::Learn { .. } => {
                                need(format!("{}.{i}.title", l.id));
                                need(format!("{}.{i}.body", l.id));
                            }
                            _ => {
                                need(format!("{}.{i}.prompt", l.id));
                                need(format!("{}.{i}.explanation", l.id));
                            }
                        }
                    }
                }
            }
        }
        let shown: Vec<_> = missing.iter().take(40).cloned().collect();
        assert!(missing.is_empty(), "{} missing Hebrew strings, e.g.:\n{}", missing.len(), shown.join("\n"));
    }
}
