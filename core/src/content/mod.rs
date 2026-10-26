//! The built-in course catalog. Every course and every lesson is open:
//! nothing here (or anywhere else) locks content behind progress or payment.
//!
//! Each course is a full curriculum in its own file. Step helpers:
//! `learn` (teaching card), `mc` (multiple choice), `fill` (tap tiles into
//! `___` blanks), `order` (lines given in the CORRECT order; shuffled when
//! served) and `typein`.

mod arduino;
mod arduino_lang;
mod javascript;
mod python;
mod rust;
mod sql;
mod web;

use std::sync::OnceLock;

use crate::model::{Course, Lesson, Section, Step};

pub(crate) fn learn(title: &'static str, body: &'static str, code: Option<&'static str>) -> Step {
    Step::Learn { title, body, code }
}

pub(crate) fn mc(
    prompt: &'static str,
    code: Option<&'static str>,
    options: &[&'static str],
    correct: usize,
    explanation: &'static str,
) -> Step {
    Step::MultipleChoice { prompt, code, options: options.to_vec(), correct, explanation }
}

pub(crate) fn fill(
    prompt: &'static str,
    code: &'static str,
    options: &[&'static str],
    answers: &[&'static str],
    explanation: &'static str,
) -> Step {
    Step::FillBlanks { prompt, code, options: options.to_vec(), answers: answers.to_vec(), explanation }
}

pub(crate) fn order(prompt: &'static str, lines: &[&'static str], explanation: &'static str) -> Step {
    Step::Reorder { prompt, lines: lines.to_vec(), explanation }
}

pub(crate) fn typein(
    prompt: &'static str,
    code: Option<&'static str>,
    accepted: &[&'static str],
    explanation: &'static str,
) -> Step {
    Step::TypeIn { prompt, code, accepted: accepted.to_vec(), explanation }
}

pub(crate) fn lesson(id: &'static str, title: &'static str, steps: Vec<Step>) -> Lesson {
    Lesson { id, title, steps }
}

pub(crate) fn section(id: &'static str, title: &'static str, lessons: Vec<Lesson>) -> Section {
    Section { id, title, lessons }
}

/// All courses, built once.
pub fn catalog() -> &'static [Course] {
    static CATALOG: OnceLock<Vec<Course>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        vec![
            python::course(),
            javascript::course(),
            web::course(),
            sql::course(),
            rust::course(),
            arduino::course(),
        ]
    })
}

pub fn find_course(id: &str) -> Option<&'static Course> {
    catalog().iter().find(|c| c.id == id)
}

/// Find a lesson and its course. Also returns the id of the lesson that
/// follows it in the course, if any.
pub fn find_lesson(id: &str) -> Option<(&'static Course, &'static Lesson, Option<&'static str>)> {
    for course in catalog() {
        let lessons: Vec<&Lesson> = course.lessons().collect();
        if let Some(pos) = lessons.iter().position(|l| l.id == id) {
            let next = lessons.get(pos + 1).map(|l| l.id);
            return Some((course, lessons[pos], next));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BLANK;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique() {
        let mut seen = HashSet::new();
        for c in catalog() {
            assert!(seen.insert(c.id), "duplicate id {}", c.id);
            for s in &c.sections {
                assert!(seen.insert(s.id), "duplicate id {}", s.id);
                for l in &s.lessons {
                    assert!(seen.insert(l.id), "duplicate id {}", l.id);
                }
            }
        }
    }

    #[test]
    fn every_step_is_well_formed() {
        for c in catalog() {
            for l in c.lessons() {
                assert!(l.exercise_count() > 0, "{} has no exercises", l.id);
                for (i, step) in l.steps.iter().enumerate() {
                    let at = format!("{} step {}", l.id, i);
                    match step {
                        Step::Learn { title, body, .. } => {
                            assert!(!title.is_empty() && !body.is_empty(), "{at}");
                        }
                        Step::MultipleChoice { options, correct, .. } => {
                            assert!(options.len() >= 2, "{at}");
                            assert!(*correct < options.len(), "{at}");
                            let unique: HashSet<_> = options.iter().collect();
                            assert_eq!(unique.len(), options.len(), "{at}: duplicate option");
                        }
                        Step::FillBlanks { code, options, answers, .. } => {
                            assert!(!code.contains("____"), "{at}: blank touches an underscore");
                            assert_eq!(code.matches(BLANK).count(), answers.len(), "{at}: blanks");
                            for a in answers {
                                let need = answers.iter().filter(|x| *x == a).count();
                                let have = options.iter().filter(|x| *x == a).count();
                                assert!(have >= need, "{at}: missing tile {a}");
                            }
                            assert!(options.len() > answers.len(), "{at}: needs distractor tiles");
                        }
                        Step::Reorder { lines, .. } => {
                            assert!(lines.len() >= 2, "{at}");
                            let unique: HashSet<_> = lines.iter().collect();
                            assert_eq!(unique.len(), lines.len(), "{at}: duplicate lines are ambiguous");
                        }
                        Step::TypeIn { accepted, .. } => assert!(!accepted.is_empty(), "{at}"),
                    }
                }
            }
        }
    }

    #[test]
    fn courses_are_full_curricula() {
        for c in catalog() {
            assert!(c.sections.len() >= 8, "{} has only {} sections", c.id, c.sections.len());
            assert!(c.lesson_count() >= 25, "{} has only {} lessons", c.id, c.lesson_count());
        }
    }

    #[test]
    fn lookup_and_next() {
        let course = find_course("python").unwrap();
        let first = course.sections[0].lessons.last().unwrap();
        let (_, _, next) = find_lesson(first.id).unwrap();
        // crossing into the next section
        assert_eq!(next, Some(course.sections[1].lessons[0].id));
        let last = course.lessons().last().unwrap();
        assert_eq!(find_lesson(last.id).unwrap().2, None);
        assert!(find_lesson("nope").is_none());
    }
}
