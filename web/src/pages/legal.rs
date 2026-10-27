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
    updated: "Last updated: October 27, 2026",
    sections: &[
        ("What Code-Iot is", "Code-Iot is an app for learning to code. The first topics of every course are open to everyone; the rest of a course is opened by a purchase (see the Terms of use)."),
        ("What we store", "If you create an account we store your username, a securely hashed password (never the password itself), your learning progress (finished lessons, XP, streaks and badges) and the profile picture you choose. If you sign in with Google we receive your Google account id, name and e-mail address from Google and use them only to create and recognise your account. Code you write in the Playground stays on your device, except Python, C++, Rust and Arduino code, which is sent to a code-running service (Compiler Explorer) only to run it and show you the result."),
        ("Purchases", "Payments are handled by Lemon Squeezy, which acts as the seller (merchant of record) and processes your card or PayPal details; we never see or store them. From Lemon Squeezy we receive and keep a record of each purchase: the course, plan, amount, status, refunds, your name and e-mail, and for subscriptions the card brand and last four digits. Before your first purchase we ask for your phone number and country so we can contact you about your purchase if needed."),
        ("What we don't do", "We don't sell your data, show ads or share your information with anyone except the services that run the app: Render (hosting), Turso (database), Google (sign-in), Lemon Squeezy (payments) and Compiler Explorer (only to run Playground code you send)."),
        ("Your device", "The app keeps a sign-in token and a few preferences (language, chosen course, Playground code) in your browser's local storage so you stay signed in."),
        ("Your choices", "You can use the app as a guest without an account, change or remove your profile picture at any time, and sign out from the Profile page. To have your account deleted, e-mail tamirmaidani@gmail.com. Purchase records may be kept as long as the law requires."),
        ("Children", "Code-Iot is meant for learners of all ages. We collect only what is needed to save learning progress."),
        ("Changes", "If this policy changes, the new version will be published on this page."),
    ],
};

const PRIVACY_HE: Doc = Doc {
    title: "מדיניות פרטיות",
    updated: "עודכן לאחרונה: 27 באוקטובר 2026",
    sections: &[
        ("מה זה Code-Iot", "Code-Iot היא אפליקציה ללימוד תכנות. הנושאים הראשונים בכל מסלול פתוחים לכולם, ושאר המסלול נפתח ברכישה (ראו תנאי השימוש)."),
        ("מה אנחנו שומרים", "אם יוצרים חשבון, נשמרים שם המשתמש, הסיסמה בצורה מוצפנת (לעולם לא הסיסמה עצמה), ההתקדמות בלימוד (שיעורים שהושלמו, XP, רצפים ותגים) ותמונת הפרופיל שבחרתם. בכניסה עם Google אנחנו מקבלים מ-Google את מזהה החשבון, השם וכתובת המייל, ומשתמשים בהם רק כדי ליצור ולזהות את החשבון שלכם. קוד שכותבים במגרש המשחקים נשאר במכשיר, חוץ מקוד Python,‏ C++,‏ Rust ו-Arduino שנשלח לשירות הרצה (Compiler Explorer) רק כדי להריץ אותו ולהציג את התוצאה."),
        ("רכישות", "התשלומים מטופלים על ידי Lemon Squeezy, שהיא המוכרת הרשמית (Merchant of Record) ומעבדת את פרטי הכרטיס או ה-PayPal שלכם. אנחנו לא רואים ולא שומרים אותם. מ-Lemon Squeezy אנחנו מקבלים ושומרים רישום של כל רכישה: המסלול, התוכנית, הסכום, הסטטוס, החזרים, השם והמייל, ובמנויים גם סוג הכרטיס וארבע הספרות האחרונות. לפני הרכישה הראשונה אנחנו מבקשים מספר טלפון ומדינה, כדי שנוכל לחזור אליכם בנוגע לרכישה אם יהיה צורך."),
        ("מה אנחנו לא עושים", "אנחנו לא מוכרים מידע, לא מציגים פרסומות ולא משתפים מידע עם אף אחד, חוץ מהשירותים שמפעילים את האפליקציה: Render (אחסון), Turso (מסד נתונים), Google (כניסה לחשבון), Lemon Squeezy (תשלומים) ו-Compiler Explorer (רק כדי להריץ קוד שאתם שולחים)."),
        ("המכשיר שלכם", "האפליקציה שומרת באחסון המקומי של הדפדפן אסימון כניסה וכמה העדפות (שפה, הקורס שנבחר, הקוד במגרש המשחקים) כדי שתישארו מחוברים."),
        ("הבחירות שלכם", "אפשר להשתמש באפליקציה כאורחים בלי חשבון, להחליף או להסיר את תמונת הפרופיל בכל זמן ולהתנתק מעמוד הפרופיל. כדי למחוק את החשבון, כתבו ל-tamirmaidani@gmail.com. רישומי רכישות עשויים להישמר כל עוד החוק מחייב."),
        ("ילדים", "Code-Iot מיועדת ללומדים בכל גיל. אנחנו אוספים רק את מה שצריך כדי לשמור את ההתקדמות בלימוד."),
        ("שינויים", "אם המדיניות תשתנה, הגרסה החדשה תפורסם בעמוד הזה."),
    ],
};

const TERMS_EN: Doc = Doc {
    title: "Terms of use",
    updated: "Last updated: October 27, 2026",
    sections: &[
        ("Using Code-Iot", "Code-Iot is an app for learning to code. Please use it respectfully: don't try to break the service, overload it or access other people's accounts."),
        ("Your account", "Keep your password to yourself. You are responsible for what happens in your account. Usernames must not be offensive or pretend to be someone else."),
        ("Free and paid content", "The first topic of every course is open to everyone. Signing in with Google opens the second topic as well. The rest of a course is opened by a purchase that applies to that course only: $5 per month (a subscription) or $25 once (access for as long as Code-Iot operates). Prices are in US dollars; sales tax or VAT may be added at checkout depending on where you live."),
        ("Payments", "Payments are processed by Lemon Squeezy, which is the seller (merchant of record) for every purchase; its terms also apply to the payment. You receive a receipt by e-mail. We never see your card details."),
        ("Subscriptions", "A subscription renews every month until you cancel it. You can cancel at any time with \"Manage subscription\" on your Profile page or by e-mailing us. After cancelling, the course stays open until the end of the month you already paid for, and you will not be charged again."),
        ("Refunds", "If you are not happy with a purchase, e-mail us within 14 days of the payment and we will refund it in full. Later requests are handled case by case. After a full refund the paid part of the course is locked again. This does not limit any rights you have under consumer protection law."),
        ("Playground code", "Code you run in the Playground must not be harmful (for example malware or attacks on other systems). Code is run by an outside service with time limits."),
        ("Content", "The lessons are provided as-is for education. We try to keep them accurate, but mistakes can happen."),
        ("Availability", "The service may change, pause or end, and features may be added or removed."),
        ("Contact", "Questions, cancellations and refunds: tamirmaidani@gmail.com."),
        ("Trademarks", "Programming language names and logos belong to their owners and are used only to name the language being taught; this does not imply any partnership or endorsement. HTML5 logo by W3C, CC BY 3.0. CSS logo by the CSS Next CG, CC0. C++ logo courtesy of the Standard C++ Foundation. Arduino® is a trademark of Arduino SA. Python® is a registered trademark of the Python Software Foundation. Rust is a trademark of the Rust Foundation. JavaScript is a trademark of Oracle."),
    ],
};

const TERMS_HE: Doc = Doc {
    title: "תנאי שימוש",
    updated: "עודכן לאחרונה: 27 באוקטובר 2026",
    sections: &[
        ("שימוש ב-Code-Iot", "Code-Iot היא אפליקציה ללימוד תכנות. השתמשו בה בכבוד: אל תנסו לפרוץ את השירות, להעמיס עליו או להיכנס לחשבונות של אחרים."),
        ("החשבון שלכם", "שמרו את הסיסמה לעצמכם. אתם אחראים למה שקורה בחשבון שלכם. שם המשתמש לא יכול להיות פוגעני או להתחזות למישהו אחר."),
        ("תוכן חינמי ותוכן בתשלום", "הנושא הראשון בכל מסלול פתוח לכולם. התחברות עם Google פותחת גם את הנושא השני. שאר המסלול נפתח ברכישה שחלה על אותו מסלול בלבד: 5$ לחודש (מנוי) או 25$ בתשלום אחד (גישה כל עוד Code-Iot פועלת). המחירים בדולר אמריקאי, ובתשלום עשוי להתווסף מע\"מ או מס לפי מקום המגורים."),
        ("תשלומים", "התשלומים מעובדים על ידי Lemon Squeezy, שהיא המוכרת הרשמית (Merchant of Record) בכל רכישה, ולכן גם התנאים שלה חלים על התשלום. קבלה נשלחת במייל. אנחנו לא רואים את פרטי הכרטיס."),
        ("מנויים", "מנוי מתחדש כל חודש עד שמבטלים אותו. אפשר לבטל בכל זמן דרך \"ניהול המנוי\" בעמוד הפרופיל או במייל אלינו. אחרי הביטול המסלול נשאר פתוח עד סוף החודש שכבר שולם, ולא יהיו חיובים נוספים."),
        ("החזרים", "אם אינכם מרוצים מרכישה, כתבו לנו תוך 14 יום מהתשלום ונחזיר את מלוא הסכום. בקשות מאוחרות יותר נבדקות לגופן. אחרי החזר מלא החלק בתשלום במסלול ננעל שוב. אין בכך כדי לגרוע מזכויות לפי חוק הגנת הצרכן."),
        ("קוד במגרש המשחקים", "אסור להריץ במגרש המשחקים קוד מזיק (למשל נוזקה או התקפות על מערכות אחרות). הקוד רץ בשירות חיצוני עם מגבלות זמן."),
        ("התוכן", "השיעורים ניתנים כמו שהם, לצורכי לימוד. אנחנו משתדלים שיהיו מדויקים, אבל טעויות יכולות לקרות."),
        ("זמינות", "השירות יכול להשתנות, להיעצר או להסתיים, ותכונות יכולות להתווסף או להיעלם."),
        ("סימנים מסחריים", "שמות וסמלים של שפות תכנות שייכים לבעליהם ומשמשים רק כדי לציין את השפה הנלמדת; אין בכך שותפות או תמיכה מצדם. סמל HTML5 מאת W3C, ברישיון CC BY 3.0. סמל CSS מאת CSS Next CG, ברישיון CC0. סמל C++ באדיבות Standard C++ Foundation. Arduino® הוא סימן מסחרי של Arduino SA. Python® הוא סימן מסחרי של Python Software Foundation. Rust הוא סימן מסחרי של Rust Foundation. JavaScript הוא סימן מסחרי של Oracle."),
        ("יצירת קשר", "שאלות, ביטולים והחזרים: tamirmaidani@gmail.com."),
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
