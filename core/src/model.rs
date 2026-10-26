//! The shape of course content. Content is compiled into the binary as
//! `'static` data (see [`crate::content`]).

/// A course such as "Python" or "HTML & CSS".
#[derive(Debug, Clone)]
pub struct Course {
    pub id: &'static str,
    pub title: &'static str,
    pub language: &'static str,
    pub description: &'static str,
    /// Accent color used by the UI (CSS color).
    pub color: &'static str,
    /// Short emoji/icon shown on the course card.
    pub icon: &'static str,
    pub sections: Vec<Section>,
}

/// A group of lessons inside a course.
#[derive(Debug, Clone)]
pub struct Section {
    pub id: &'static str,
    pub title: &'static str,
    pub lessons: Vec<Lesson>,
}

/// A bite-sized lesson made of a few steps.
#[derive(Debug, Clone)]
pub struct Lesson {
    pub id: &'static str,
    pub title: &'static str,
    pub steps: Vec<Step>,
}

/// Marker used inside `FillBlanks` code for each blank.
pub const BLANK: &str = "___";

/// One screen of a lesson.
#[derive(Debug, Clone)]
pub enum Step {
    /// A teaching card. Not graded.
    Learn {
        title: &'static str,
        body: &'static str,
        code: Option<&'static str>,
    },
    /// Pick the single correct option.
    MultipleChoice {
        prompt: &'static str,
        code: Option<&'static str>,
        options: Vec<&'static str>,
        correct: usize,
        explanation: &'static str,
    },
    /// Fill each `___` in `code` by tapping tiles from `options`.
    FillBlanks {
        prompt: &'static str,
        code: &'static str,
        options: Vec<&'static str>,
        answers: Vec<&'static str>,
        explanation: &'static str,
    },
    /// Put the lines in the right order. `lines` is the correct order.
    Reorder {
        prompt: &'static str,
        lines: Vec<&'static str>,
        explanation: &'static str,
    },
    /// Type the answer. Any of `accepted` is correct (whitespace and quote
    /// style are ignored).
    TypeIn {
        prompt: &'static str,
        code: Option<&'static str>,
        accepted: Vec<&'static str>,
        explanation: &'static str,
    },
}

impl Step {
    /// Whether the step is graded (everything except `Learn`).
    pub fn is_exercise(&self) -> bool {
        !matches!(self, Step::Learn { .. })
    }
}

impl Lesson {
    /// Number of graded steps in the lesson.
    pub fn exercise_count(&self) -> usize {
        self.steps.iter().filter(|s| s.is_exercise()).count()
    }
}

impl Course {
    pub fn lessons(&self) -> impl Iterator<Item = &Lesson> {
        self.sections.iter().flat_map(|s| s.lessons.iter())
    }

    pub fn lesson_count(&self) -> usize {
        self.lessons().count()
    }
}
