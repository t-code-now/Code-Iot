//! Interface strings in English and Hebrew. Course content is translated
//! on the server (see `codesprout_core::i18n`); this file covers the app UI.

pub use codesprout_core::i18n::Lang;

/// (key, English, Hebrew). `{}` placeholders are filled in order by [`tf`].
const STRINGS: &[(&str, &str, &str)] = &[
    // tabs & navigation
    ("tab.learn", "Learn", "לימוד"),
    ("tab.playground", "Playground", "מגרש משחקים"),
    ("tab.league", "League", "ליגה"),
    ("tab.profile", "Profile", "פרופיל"),
    ("back", "‹ Back", "› חזרה"),
    ("back.arrow", "‹", "›"),
    ("back.learn", "Back to Learn", "חזרה ללימוד"),
    // home / courses
    ("home.lessons_progress", "{} of {} lessons", "{} מתוך {} שיעורים"),
    ("home.section", "Section {}", "פרק {}"),
    ("home.start", "START", "התחל"),
    ("home.finish", "🏁 Finish line — complete every section to master the course", "🏁 קו הסיום — השלימו את כל הפרקים כדי לשלוט בקורס"),
    ("courses.title", "Courses", "קורסים"),
    ("courses.count", "{} / {} lessons", "{} / {} שיעורים"),
    // lesson player
    ("lesson.close", "Close lesson", "סגירת השיעור"),
    ("lesson.learn_badge", "Learn", "לימוד"),
    ("lesson.check", "Check", "בדיקה"),
    ("lesson.checking", "Checking…", "בודק…"),
    ("lesson.continue", "Continue", "המשך"),
    ("lesson.not_quite", "Not quite", "לא בדיוק"),
    ("lesson.correct_answer", "Correct answer:", "התשובה הנכונה:"),
    ("lesson.reorder_hint", "Tap the lines below in the right order", "הקישו על השורות שלמטה בסדר הנכון"),
    ("lesson.type_placeholder", "Type your answer…", "הקלידו את התשובה…"),
    ("praise.0", "Nice!", "יפה!"),
    ("praise.1", "Great job!", "עבודה מצוינת!"),
    ("praise.2", "Correct!", "נכון!"),
    ("praise.3", "You got it!", "הצלחתם!"),
    ("praise.4", "Awesome!", "מדהים!"),
    // lesson complete
    ("done.title", "Lesson complete!", "השיעור הושלם!"),
    ("done.xp", "XP", "XP"),
    ("done.accuracy", "Accuracy", "דיוק"),
    ("done.streak", "Streak", "רצף"),
    ("done.new_badge", "New badge unlocked!", "תג חדש נפתח!"),
    ("done.not_saved", "Progress wasn't saved: {}", "ההתקדמות לא נשמרה: {}"),
    ("done.guest_note", "Create a free account to keep your XP, streak and badges.", "צרו חשבון חינמי כדי לשמור את ה-XP, הרצף והתגים שלכם."),
    ("done.guest_button", "Sign up to save progress", "הרשמה לשמירת ההתקדמות"),
    ("done.next", "Next lesson", "השיעור הבא"),
    ("done.back", "Back to path", "חזרה למסלול"),
    // welcome & auth
    ("welcome.sub", "Learn to code in bite-sized lessons. Every course is open — start anywhere.", "למדו לתכנת בשיעורים קצרים. כל הקורסים פתוחים — התחילו מאיפה שתרצו."),
    ("welcome.start", "Get started", "בואו נתחיל"),
    ("welcome.have_account", "I already have an account", "כבר יש לי חשבון"),
    ("welcome.guest", "Try it without an account", "לנסות בלי חשבון"),
    ("auth.signup_title", "Create your account", "יצירת חשבון"),
    ("auth.login_title", "Welcome back", "ברוכים השבים"),
    ("auth.signup", "Sign up", "הרשמה"),
    ("auth.login", "Log in", "התחברות"),
    ("auth.have_account", "Already have an account?", "כבר יש לכם חשבון?"),
    ("auth.new_here", "New here?", "חדשים כאן?"),
    ("auth.create_account", "Create an account", "יצירת חשבון"),
    ("auth.username", "Username", "שם משתמש"),
    ("auth.username_ph", "e.g. code_ninja", "למשל code_ninja"),
    ("auth.password", "Password", "סיסמה"),
    ("auth.password_ph", "At least 6 characters", "לפחות 6 תווים"),
    ("auth.wait", "Please wait…", "רק רגע…"),
    // league
    ("league.title", "League", "ליגה"),
    ("league.name", "Sprout League", "ליגת הנבטים"),
    ("league.desc", "Earn XP from lessons to climb. The weekly board resets every Monday.", "צברו XP בשיעורים כדי לטפס. הטבלה השבועית מתאפסת בכל יום שני."),
    ("league.week", "This week", "השבוע"),
    ("league.all", "All time", "כל הזמנים"),
    ("league.empty", "No XP earned yet. Finish a lesson to take first place!", "עוד לא נצבר XP. סיימו שיעור כדי לתפוס את המקום הראשון!"),
    ("league.you", "you", "את/ה"),
    ("league.xp", "{} XP", "{} XP"),
    ("league.guest", "Sign up to appear on the leaderboard.", "הירשמו כדי להופיע בטבלת המובילים."),
    ("league.create", "Create account", "יצירת חשבון"),
    // profile
    ("profile.title", "Profile", "פרופיל"),
    ("profile.joined", "Joined {}", "הצטרפות: {}"),
    ("profile.level", "Level {}", "רמה {}"),
    ("profile.level_xp", "{} / {} XP", "{} / {} XP"),
    ("profile.day_streak", "Day streak", "ימי רצף"),
    ("profile.total_xp", "Total XP", "סה״כ XP"),
    ("profile.week_xp", "XP this week", "XP השבוע"),
    ("profile.best_streak", "Best streak", "הרצף הטוב ביותר"),
    ("profile.lessons", "Lessons", "שיעורים"),
    ("profile.courses", "Courses", "קורסים"),
    ("profile.badges", "Badges", "תגים"),
    ("profile.logout", "Log out", "התנתקות"),
    ("profile.guest", "Guest", "אורח"),
    ("profile.guest_desc", "You're learning without an account. Your progress isn't saved.", "אתם לומדים בלי חשבון. ההתקדמות שלכם לא נשמרת."),
    ("profile.create_free", "Create a free account", "יצירת חשבון חינמי"),
    ("profile.language", "Language", "שפה"),
    // playground
    ("pg.title", "Playground", "מגרש משחקים"),
    ("pg.reset", "Reset", "איפוס"),
    ("pg.run", "Run", "הרצה"),
    ("pg.result", "Result", "תוצאה"),
    ("pg.tip", "Tip: code runs in a sandbox right in your browser.", "טיפ: הקוד רץ בסביבה מבודדת ישירות בדפדפן."),
    // badges (by id)
    ("badge.first_lesson.title", "First Steps", "צעדים ראשונים"),
    ("badge.first_lesson.desc", "Finish your first lesson", "סיימו את השיעור הראשון"),
    ("badge.ten_lessons.title", "On a Roll", "בתנופה"),
    ("badge.ten_lessons.desc", "Finish 10 lessons", "סיימו 10 שיעורים"),
    ("badge.perfect.title", "Flawless", "ללא רבב"),
    ("badge.perfect.desc", "Finish a lesson with every answer right", "סיימו שיעור עם כל התשובות נכונות"),
    ("badge.xp_100.title", "Century", "מאה"),
    ("badge.xp_100.desc", "Earn 100 XP", "צברו 100 XP"),
    ("badge.xp_1000.title", "Powerhouse", "תחנת כוח"),
    ("badge.xp_1000.desc", "Earn 1,000 XP", "צברו 1,000 XP"),
    ("badge.streak_3.title", "Warming Up", "מתחממים"),
    ("badge.streak_3.desc", "Reach a 3-day streak", "הגיעו לרצף של 3 ימים"),
    ("badge.streak_7.title", "Week Warrior", "לוחמי השבוע"),
    ("badge.streak_7.desc", "Reach a 7-day streak", "הגיעו לרצף של 7 ימים"),
    ("badge.course_done.title", "Graduate", "בוגרים"),
    ("badge.course_done.desc", "Complete every lesson in a course", "השלימו את כל השיעורים בקורס"),
    // server messages (matched by their English text in `error`)
    ("err.username", "Username must be 3–20 letters, numbers or underscores", "שם המשתמש חייב להכיל 3–20 אותיות, ספרות או קווים תחתונים"),
    ("err.password", "Password must be at least 6 characters", "הסיסמה חייבת להכיל לפחות 6 תווים"),
    ("err.taken", "That username is taken", "שם המשתמש הזה תפוס"),
    ("err.wrong", "Wrong username or password", "שם משתמש או סיסמה שגויים"),
    ("err.login", "Please log in", "נא להתחבר"),
    ("err.generic", "Something went wrong", "משהו השתבש"),
    ("err.network", "Can't reach the server", "אין חיבור לשרת"),
];

/// The UI string for `key`. Unknown keys return the key itself, which makes
/// missing strings easy to spot.
pub fn t(lang: Lang, key: &'static str) -> &'static str {
    STRINGS
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, en, he)| if lang == Lang::He { *he } else { *en })
        .unwrap_or(key)
}

/// Look up a key built at runtime (e.g. "badge.<id>.title").
pub fn lookup(lang: Lang, key: &str) -> Option<&'static str> {
    STRINGS
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, en, he)| if lang == Lang::He { *he } else { *en })
}

/// A badge's title and description in the current language.
pub fn badge(lang: Lang, b: &codesprout_core::api::Badge) -> (String, String) {
    let title = lookup(lang, &format!("badge.{}.title", b.id)).map(str::to_string).unwrap_or_else(|| b.title.clone());
    let desc = lookup(lang, &format!("badge.{}.desc", b.id)).map(str::to_string).unwrap_or_else(|| b.description.clone());
    (title, desc)
}

/// Like [`t`], filling each `{}` with the next argument.
pub fn tf(lang: Lang, key: &'static str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = String::new();
    let mut rest = t(lang, key);
    for arg in args {
        match rest.split_once("{}") {
            Some((before, after)) => {
                out.push_str(before);
                out.push_str(&arg.to_string());
                rest = after;
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}

/// Translate a server error message (sent in English) when we know it.
pub fn error(lang: Lang, message: &str) -> String {
    if lang == Lang::En {
        return message.to_string();
    }
    for (key, en, he) in STRINGS {
        if key.starts_with("err.") && message.starts_with(en) {
            return he.to_string();
        }
    }
    message.to_string()
}

/// Letters used to label answer options.
pub fn option_letter(lang: Lang, i: usize) -> String {
    const HE: [char; 6] = ['א', 'ב', 'ג', 'ד', 'ה', 'ו'];
    match lang {
        Lang::He => HE.get(i).copied().unwrap_or('•').to_string(),
        Lang::En => ((b'A' + i as u8) as char).to_string(),
    }
}

/// Locale tag for dates and numbers.
pub fn locale(lang: Lang) -> &'static str {
    match lang {
        Lang::He => "he-IL",
        Lang::En => "en-US",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn keys_unique_and_complete() {
        let mut seen = HashSet::new();
        for (k, en, he) in STRINGS {
            assert!(seen.insert(*k), "duplicate key {k}");
            assert!(!en.is_empty() && !he.is_empty(), "{k}");
            assert_eq!(en.matches("{}").count(), he.matches("{}").count(), "placeholders differ in {k}");
        }
    }

    #[test]
    fn formatting() {
        assert_eq!(tf(Lang::En, "home.lessons_progress", &[&3, &10]), "3 of 10 lessons");
        assert_eq!(tf(Lang::He, "home.section", &[&2]), "פרק 2");
        assert_eq!(t(Lang::En, "nope"), "nope");
        assert_eq!(error(Lang::He, "That username is taken"), "שם המשתמש הזה תפוס");
        assert_eq!(error(Lang::He, "Something else"), "Something else");
        assert_eq!(option_letter(Lang::He, 1), "ב");
        assert_eq!(option_letter(Lang::En, 2), "C");
    }
}
