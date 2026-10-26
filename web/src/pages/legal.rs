//! Privacy policy and terms of use (linked from Google's sign-in screen).

use leptos::*;

use crate::i18n::Lang;
use crate::session::use_session;

struct Doc {
    title: &'static str,
    updated: &'static str,
    sections: &'static [(&'static str, &'static str)],
}

const PRIVACY_EN: Doc = Doc {
    title: "Privacy policy",
    updated: "Last updated: October 26, 2026",
    sections: &[
        ("What Code-Iot is", "Code-Iot is a free app for learning to code. Every course and lesson is open to everyone, with or without an account."),
        ("What we store", "If you create an account we store your username, a securely hashed password (never the password itself), your learning progress (finished lessons, XP, streaks and badges) and the profile picture you choose. If you sign in with Google we receive your Google account id, name and e-mail address from Google and use them only to create and recognise your account. Code you write in the Playground stays on your device, except Python, C++, Rust and Arduino code, which is sent to a code-running service (Compiler Explorer) only to run it and show you the result."),
        ("What we don't do", "We don't sell your data, show ads or share your information with anyone except the services that run the app: Render (hosting), Turso (database), Google (only when you choose to sign in with Google) and Compiler Explorer (only to run Playground code you send)."),
        ("Your device", "The app keeps a sign-in token and a few preferences (language, chosen course, Playground code) in your browser's local storage so you stay signed in."),
        ("Your choices", "You can use the app as a guest without an account, change or remove your profile picture at any time, and sign out from the Profile page. To have your account deleted, open an issue at github.com/t-code-now/Code-Iot."),
        ("Children", "Code-Iot is meant for learners of all ages. We collect only what is needed to save learning progress."),
        ("Changes", "If this policy changes, the new version will be published on this page."),
    ],
};

const PRIVACY_HE: Doc = Doc {
    title: "מדיניות פרטיות",
    updated: "עודכן לאחרונה: 26 באוקטובר 2026",
    sections: &[
        ("מה זה Code-Iot", "Code-Iot היא אפליקציה חינמית ללימוד תכנות. כל הקורסים וכל השיעורים פתוחים לכולם, עם חשבון או בלי."),
        ("מה אנחנו שומרים", "אם יוצרים חשבון, נשמרים שם המשתמש, הסיסמה בצורה מוצפנת (לעולם לא הסיסמה עצמה), ההתקדמות בלימוד (שיעורים שהושלמו, XP, רצפים ותגים) ותמונת הפרופיל שבחרתם. בכניסה עם Google אנחנו מקבלים מ-Google את מזהה החשבון, השם וכתובת המייל, ומשתמשים בהם רק כדי ליצור ולזהות את החשבון שלכם. קוד שכותבים במגרש המשחקים נשאר במכשיר, חוץ מקוד Python,‏ C++,‏ Rust ו-Arduino שנשלח לשירות הרצה (Compiler Explorer) רק כדי להריץ אותו ולהציג את התוצאה."),
        ("מה אנחנו לא עושים", "אנחנו לא מוכרים מידע, לא מציגים פרסומות ולא משתפים מידע עם אף אחד, חוץ מהשירותים שמפעילים את האפליקציה: Render (אחסון), Turso (מסד נתונים), Google (רק אם בחרתם להיכנס עם Google) ו-Compiler Explorer (רק כדי להריץ קוד שאתם שולחים)."),
        ("המכשיר שלכם", "האפליקציה שומרת באחסון המקומי של הדפדפן אסימון כניסה וכמה העדפות (שפה, הקורס שנבחר, הקוד במגרש המשחקים) כדי שתישארו מחוברים."),
        ("הבחירות שלכם", "אפשר להשתמש באפליקציה כאורחים בלי חשבון, להחליף או להסיר את תמונת הפרופיל בכל זמן ולהתנתק מעמוד הפרופיל. כדי למחוק את החשבון, פתחו בקשה ב-github.com/t-code-now/Code-Iot."),
        ("ילדים", "Code-Iot מיועדת ללומדים בכל גיל. אנחנו אוספים רק את מה שצריך כדי לשמור את ההתקדמות בלימוד."),
        ("שינויים", "אם המדיניות תשתנה, הגרסה החדשה תפורסם בעמוד הזה."),
    ],
};

const TERMS_EN: Doc = Doc {
    title: "Terms of use",
    updated: "Last updated: October 26, 2026",
    sections: &[
        ("Using Code-Iot", "Code-Iot is free to use for learning. Please use it respectfully: don't try to break the service, overload it or access other people's accounts."),
        ("Your account", "Keep your password to yourself. You are responsible for what happens in your account. Usernames must not be offensive or pretend to be someone else."),
        ("Playground code", "Code you run in the Playground must not be harmful (for example malware or attacks on other systems). Code is run by an outside service with time limits."),
        ("Content", "The lessons are provided as-is for education. We try to keep them accurate, but mistakes can happen."),
        ("Availability", "The service may change, pause or end, and features may be added or removed."),
        ("Contact", "Questions or requests: github.com/t-code-now/Code-Iot."),
    ],
};

const TERMS_HE: Doc = Doc {
    title: "תנאי שימוש",
    updated: "עודכן לאחרונה: 26 באוקטובר 2026",
    sections: &[
        ("שימוש ב-Code-Iot", "השימוש ב-Code-Iot ללימוד הוא בחינם. השתמשו בה בכבוד: אל תנסו לפרוץ את השירות, להעמיס עליו או להיכנס לחשבונות של אחרים."),
        ("החשבון שלכם", "שמרו את הסיסמה לעצמכם. אתם אחראים למה שקורה בחשבון שלכם. שם המשתמש לא יכול להיות פוגעני או להתחזות למישהו אחר."),
        ("קוד במגרש המשחקים", "אסור להריץ במגרש המשחקים קוד מזיק (למשל נוזקה או התקפות על מערכות אחרות). הקוד רץ בשירות חיצוני עם מגבלות זמן."),
        ("התוכן", "השיעורים ניתנים כמו שהם, לצורכי לימוד. אנחנו משתדלים שיהיו מדויקים, אבל טעויות יכולות לקרות."),
        ("זמינות", "השירות יכול להשתנות, להיעצר או להסתיים, ותכונות יכולות להתווסף או להיעלם."),
        ("יצירת קשר", "שאלות ובקשות: github.com/t-code-now/Code-Iot."),
    ],
};

fn render(doc: &'static Doc) -> impl IntoView {
    view! {
        <div class="auth legal">
            <a href="/" class="back-link">{use_session().t("back")}</a>
            <h1 class="auth-title">{doc.title}</h1>
            <div class="legal-card">
                <p class="muted">{doc.updated}</p>
                {doc.sections.iter().map(|(h, body)| view! {
                    <h2>{*h}</h2>
                    <p>{*body}</p>
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
pub fn PrivacyPage() -> impl IntoView {
    let session = use_session();
    move || render(if session.lang.get() == Lang::He { &PRIVACY_HE } else { &PRIVACY_EN })
}

#[component]
pub fn TermsPage() -> impl IntoView {
    let session = use_session();
    move || render(if session.lang.get() == Lang::He { &TERMS_HE } else { &TERMS_EN })
}
