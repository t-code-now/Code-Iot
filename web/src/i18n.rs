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
    ("home.continue", "Continue learning", "המשך ללמוד"),
    ("home.all_courses", "All courses", "כל הקורסים"),
    ("home.league_cta", "See where you stand this week", "בדקו איפה אתם עומדים השבוע"),
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
    ("done.guest_note", "Create an account to keep your XP, streak and badges.", "צרו חשבון כדי לשמור את ה-XP, הרצף והתגים שלכם."),
    ("done.guest_button", "Sign up to save progress", "הרשמה לשמירת ההתקדמות"),
    ("done.next", "Next lesson", "השיעור הבא"),
    ("done.back", "Back to path", "חזרה למסלול"),
    // welcome & auth
    ("welcome.sub", "Learn to code anywhere, anytime!", "ללמוד לתכנת בכל מקום, בכל זמן!"),
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
    ("league.name", "Code-Iot League", "ליגת Code-Iot"),
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
    ("profile.create_free", "Create an account", "יצירת חשבון"),
    ("profile.language", "Language", "שפה"),
    ("legal.privacy", "Privacy", "פרטיות"),
    ("legal.terms", "Terms", "תנאי שימוש"),
    ("auth.google", "Continue with Google", "המשך עם Google"),
    ("unlock.phone_why", "Leave a phone number so we can reach you about your purchase if needed:", "השאירו מספר טלפון כדי שנוכל לחזור אליכם בנוגע לרכישה אם יהיה צורך:"),
    ("unlock.phone_save", "Continue", "המשך"),
    ("unlock.phone_change", "Change", "שינוי"),
    ("admin.refunded", "Refunded", "הוחזר"),
    ("admin.pause_confirm", "Stop all new purchases? People who already paid keep their courses.", "לעצור את כל הרכישות החדשות? מי שכבר שילם ממשיך ליהנות מהמסלולים שלו."),
    ("admin.resume_confirm", "Open purchases again?", "לפתוח שוב את הרכישות?"),
    ("admin.sales_paused", "Purchases are stopped", "הרכישות עצורות"),
    ("admin.sales_open", "Purchases are open", "הרכישות פתוחות"),
    ("admin.sales_hint", "Stopping blocks new checkouts only; existing subscriptions are managed per payment.", "העצירה חוסמת רק רכישות חדשות; מינויים קיימים מנוהלים בכל תשלום בנפרד."),
    ("admin.pause", "Stop purchases", "עצירת רכישות"),
    ("admin.resume", "Open purchases", "פתיחת רכישות"),
    ("admin.user_payments", "Payments", "תשלומים"),
    ("admin.f_all", "All", "הכול"),
    ("admin.f_charges", "Charges", "חיובים"),
    ("admin.f_refunds", "Refunds", "החזרים"),
    ("admin.f_subs", "Subscriptions", "מינויים"),
    ("admin.f_test", "Test", "בדיקה"),
    ("admin.pay_search_ph", "Search by e-mail, name, phone or payment id", "חיפוש לפי מייל, שם, טלפון או מספר תשלום"),
    ("admin.payments_of", "Payments of", "התשלומים של"),
    ("admin.show_all", "Show everyone", "הצגת כולם"),
    ("admin.count_payments", "Records", "רשומות"),
    ("admin.gross", "Charged", "נגבה"),
    ("admin.net", "Net", "נטו"),
    ("admin.subscription", "Subscription", "מינוי"),
    ("admin.amount_invalid", "Enter an amount like 10 or 12.50", "הזינו סכום כמו 10 או 12.50"),
    ("admin.amount_too_high", "At most {} can be refunded", "אפשר להחזיר לכל היותר {}"),
    ("admin.refund_confirm", "Refund {} to {}? The money goes back to the buyer's card and can't be taken back.", "להחזיר {} ל-{}? הכסף חוזר לכרטיס של הקונה ואי אפשר לבטל את זה."),
    ("admin.refund_done", "Refunded {}.", "הוחזרו {}."),
    ("admin.cancel_confirm", "Cancel the subscription of {}? No more charges; the course stays open until the paid period ends.", "לבטל את המינוי של {}? לא יהיו יותר חיובים; המסלול יישאר פתוח עד סוף התקופה ששולמה."),
    ("admin.cancel_done", "The subscription was cancelled.", "המינוי בוטל."),
    ("admin.d_ref", "Payment id", "מספר תשלום"),
    ("admin.d_customer", "Name on payment", "שם בתשלום"),
    ("admin.d_email", "E-mail", "מייל"),
    ("admin.d_phone", "Phone", "טלפון"),
    ("admin.d_country", "Country", "מדינה"),
    ("admin.d_user", "User id", "מזהה משתמש"),
    ("admin.d_amount", "Amount", "סכום"),
    ("admin.d_refunded", "Refunded", "הוחזר"),
    ("admin.d_refundable", "Can still refund", "אפשר עוד להחזיר"),
    ("admin.d_until", "Until", "עד"),
    ("admin.per_month", " / month", " / חודש"),
    ("cancel.title", "Cancel a purchase", "ביטול עסקה"),
    ("cancel.intro", "Here you can cancel a subscription or ask for a refund. A subscription stops charging right away and stays open until the end of the month you paid for. Any purchase can be refunded in full within 14 days.", "כאן אפשר לבטל מנוי או לבקש החזר. מנוי שמבוטל מפסיק לחייב מיד, ונשאר פתוח עד סוף החודש ששולם. כל רכישה אפשר להחזיר במלואה תוך 14 יום."),
    ("cancel.sign_in", "Sign in to see your purchases.", "התחברו כדי לראות את הרכישות שלכם."),
    ("cancel.none", "You have no purchases.", "אין לכם רכישות."),
    ("cancel.s_active", "Monthly subscription · renews on {}", "מנוי חודשי · מתחדש ב-{}"),
    ("cancel.s_cancelled", "Subscription cancelled · open until {}", "המנוי בוטל · פתוח עד {}"),
    ("cancel.s_lifetime", "Bought for good", "נרכש לתמיד"),
    ("cancel.s_refunded", "Refunded", "הוחזר"),
    ("cancel.bought", "Bought on {} · {}", "נרכש ב-{} · {}"),
    ("cancel.sub_button", "Cancel subscription", "ביטול המנוי"),
    ("cancel.refund_button", "Ask for a refund", "בקשת החזר"),
    ("cancel.contact_button", "Contact us", "פנייה אלינו"),
    ("cancel.confirm", "Cancel the {} subscription? Charges stop now; the course stays open until the end of the paid month.", "לבטל את המנוי ל-{}? החיובים ייפסקו עכשיו, והמסלול יישאר פתוח עד סוף החודש ששולם."),
    ("cancel.done", "The subscription is cancelled. No more charges. The course stays open until {}.", "המנוי בוטל. לא יהיו חיובים נוספים. המסלול פתוח עד {}."),
    ("cancel.refund_note", "Within 14 days of paying you get a full refund — just send the e-mail.", "תוך 14 יום מהתשלום מקבלים החזר מלא – פשוט שלחו את המייל."),
    ("cancel.prorata_note", "After cancelling you can also ask for a refund of the unused part of the current month.", "אחרי ביטול אפשר גם לבקש החזר על החלק שלא נוצל מהחודש הנוכחי."),
    ("cancel.mail_subject", "Refund request – {}", "בקשת החזר – {}"),
    ("cancel.mail_body", "Hello,\n\nI'd like a refund for {} (bought on {}, {}).\nPurchase number: {}\n\nThanks", "שלום,\n\nאבקש החזר עבור {} (נרכש ב-{}, {}).\nמספר רכישה: {}\n\nתודה"),
    ("cancel.other_ways", "You can also cancel or ask for a refund by e-mail:", "אפשר גם לבטל או לבקש החזר במייל:"),
    ("cancel.terms_link", "Cancellation and refund terms", "תנאי ביטול והחזרים"),
    ("status.paid", "Paid", "שולם"),
    ("status.partial_refund", "Partly refunded", "הוחזר חלקית"),
    ("status.refunded", "Refunded", "הוחזר"),
    ("status.active", "Active", "פעיל"),
    ("status.cancelled", "Cancelled", "בוטל"),
    ("status.expired", "Ended", "הסתיים"),
    ("status.past_due", "Payment failed", "תשלום נכשל"),
    ("status.unpaid", "Unpaid", "לא שולם"),
    ("status.paused", "Paused", "מושהה"),
    ("status.on_trial", "Trial", "תקופת ניסיון"),
    ("status.pending", "Pending", "ממתין"),
    ("status.failed", "Failed", "נכשל"),
    ("status.void", "Void", "בוטל"),
    ("admin.d_created", "Created", "נוצר"),
    ("admin.d_updated", "Last update", "עדכון אחרון"),
    ("admin.d_test", "Mode", "מצב"),
    ("admin.details", "Details", "פרטים"),
    ("admin.hide_details", "Hide details", "הסתרת פרטים"),
    ("admin.cancel_sub", "Cancel subscription", "ביטול מינוי"),
    ("admin.refund_amount", "Refund amount", "סכום להחזר"),
    ("admin.max_refund", "up to {}", "עד {}"),
    ("admin.full_refund", "Full amount", "הסכום המלא"),
    ("admin.refund", "Refund", "החזר"),
    ("admin.raw", "Exactly as received from Lemon Squeezy:", "בדיוק כפי שהתקבל מ-Lemon Squeezy:"),
    ("err.phone_first", "Please add your phone number first", "אנא הוסיפו קודם מספר טלפון"),
    ("err.country", "Choose your country", "בחרו מדינה"),
    ("err.phone_digits", "The phone number may contain only digits", "מספר הטלפון יכול להכיל רק ספרות"),
    ("err.phone_valid", "Please enter a valid phone number", "אנא הזינו מספר טלפון תקין"),
    ("err.nothing_left", "Nothing is left to refund on this payment", "לא נשאר מה להחזיר בתשלום הזה"),
    ("err.refund_range", "The refund must be between", "סכום ההחזר חייב להיות בין 0.01 לבין הסכום שנותר"),
    ("err.in_flight", "This payment is already being handled", "התשלום הזה כבר בטיפול"),
    ("err.not_sub", "This is not an active subscription", "זה לא מינוי פעיל"),
    ("err.not_connected", "Payments are not connected", "התשלומים לא מחוברים"),
    ("err.only_charge", "Only a charge can be refunded", "אפשר להחזיר רק חיוב"),
    ("unlock.open_course", "Open the full course", "פתיחת המסלול המלא"),
    ("unlock.opened", "{} is fully open!", "המסלול {} פתוח במלואו!"),
    ("unlock.opened_sub", "Every topic and project is yours. Enjoy!", "כל הנושאים והפרויקטים פתוחים לך. לימוד מהנה!"),
    ("unlock.open_of", "{} of {} topics open", "{} מתוך {} נושאים פתוחים"),
    ("unlock.waiting", "Confirming your payment… this screen opens the course by itself.", "מאשרים את התשלום… המסך הזה יפתח את המסלול לבד."),
    ("unlock.guest", "Sign in with Google to open the second topic of every course for free and keep your progress.", "התחברו עם Google כדי לפתוח בחינם את הנושא השני בכל מסלול ולשמור את ההתקדמות."),
    ("unlock.guest_then", "After that you can open a full course for $5 a month or $25 once.", "אחר כך אפשר לפתוח מסלול מלא ב-5$ לחודש או ב-25$ חד-פעמי."),
    ("unlock.choose", "Open every topic and project of this course:", "פתחו את כל הנושאים והפרויקטים במסלול הזה:"),
    ("unlock.soon", "Purchases open soon. Meanwhile the first topics of every course are free.", "הרכישה תיפתח בקרוב. בינתיים הנושאים הראשונים בכל מסלול פתוחים בחינם."),
    ("unlock.monthly", "Monthly", "חודשי"),
    ("unlock.per_month", "per month", "לחודש"),
    ("unlock.monthly_note", "Cancel anytime", "אפשר לבטל בכל זמן"),
    ("unlock.best", "Best value", "הכי משתלם"),
    ("unlock.lifetime", "Forever", "לתמיד"),
    ("unlock.once", "one time", "תשלום חד-פעמי"),
    ("unlock.lifetime_note", "Pay once, keep the course for good", "משלמים פעם אחת, והמסלול שלכם לתמיד"),
    ("unlock.only_this", "The purchase opens this course only. Payment is handled securely by Lemon Squeezy.", "הרכישה פותחת את המסלול הזה בלבד. התשלום מאובטח ומתבצע דרך Lemon Squeezy."),
    ("auth.signup_google", "Create your account with Google — one tap, no password to remember.", "צרו חשבון עם Google — בלחיצה אחת, בלי סיסמה לזכור."),
    ("profile.plans", "My courses", "המסלולים שלי"),
    ("profile.until", "Monthly · until {}", "חודשי · עד {}"),
    ("profile.manage", "Manage subscription", "ניהול המנוי"),
    ("admin.title", "Admin", "ניהול"),
    ("admin.denied", "This page is for the site admin only.", "הדף הזה מיועד למנהל האתר בלבד."),
    ("admin.tab_overview", "Overview", "סקירה"),
    ("admin.tab_users", "Users", "משתמשים"),
    ("admin.tab_payments", "Payments", "תשלומים"),
    ("admin.tab_log", "Log", "יומן"),
    ("admin.users", "Users", "משתמשים"),
    ("admin.active_7d", "Active this week", "פעילים השבוע"),
    ("admin.new_7d", "New this week", "חדשים השבוע"),
    ("admin.lessons_7d", "Lessons this week", "שיעורים השבוע"),
    ("admin.paying", "Paying users", "משתמשים משלמים"),
    ("admin.lifetime", "Lifetime purchases", "רכישות לתמיד"),
    ("admin.subs", "Active subscriptions", "מינויים פעילים"),
    ("admin.revenue", "Revenue", "הכנסות"),
    ("admin.google_users", "Google accounts", "חשבונות Google"),
    ("admin.banned", "Suspended", "מושעים"),
    ("admin.payments_off", "Payments are not connected yet (Lemon Squeezy settings missing in Render).", "התשלומים עדיין לא מחוברים (חסרות הגדרות Lemon Squeezy ב-Render)."),
    ("admin.by_course", "By course", "לפי מסלול"),
    ("admin.course", "Course", "מסלול"),
    ("admin.learners", "Learners", "לומדים"),
    ("admin.full_access", "Full access", "גישה מלאה"),
    ("admin.state_lifetime", "Bought forever", "נרכש לתמיד"),
    ("admin.state_monthly", "Monthly", "מינוי חודשי"),
    ("admin.state_open", "Opened by admin", "נפתח ידנית"),
    ("admin.state_blocked", "Blocked", "חסום"),
    ("admin.state_admin", "Admin", "מנהל"),
    ("admin.state_free", "Free topics", "נושאים חינמיים"),
    ("admin.search_ph", "Search by username or e-mail", "חיפוש לפי שם משתמש או מייל"),
    ("admin.search", "Search", "חיפוש"),
    ("admin.no_users", "No users found.", "לא נמצאו משתמשים."),
    ("admin.count", "{} users", "{} משתמשים"),
    ("admin.password", "Password", "סיסמה"),
    ("admin.suspended", "Suspended", "מושעה"),
    ("admin.joined", "Joined", "הצטרף"),
    ("admin.last_active", "Last active", "פעילות אחרונה"),
    ("admin.lessons", "Lessons", "שיעורים"),
    ("admin.paid", "Paid", "שילם"),
    ("admin.method", "Payment method", "אמצעי תשלום"),
    ("admin.mode_default", "By payment", "לפי תשלום"),
    ("admin.mode_open", "Open", "פתוח"),
    ("admin.mode_block", "Block", "חסום"),
    ("admin.restore", "Restore account", "ביטול השעיה"),
    ("admin.suspend", "Suspend account", "השעיית חשבון"),
    ("admin.delete", "Delete account", "מחיקת חשבון"),
    ("admin.delete_confirm", "Delete the account “{}” and all its progress? This can't be undone.", "למחוק את החשבון “{}” ואת כל ההתקדמות שלו? אי אפשר לבטל את זה."),
    ("admin.no_payments", "No payments yet.", "עדיין אין תשלומים."),
    ("admin.refund_hint", "Refund any amount or cancel a subscription right here. Every action asks for confirmation and is written to the log.", "אפשר להחזיר כל סכום או לבטל מינוי ישירות מכאן. כל פעולה מבקשת אישור ונרשמת ביומן."),
    ("admin.date", "Date", "תאריך"),
    ("admin.user", "User", "משתמש"),
    ("admin.plan", "Plan", "תוכנית"),
    ("admin.status", "Status", "סטטוס"),
    ("admin.amount", "Amount", "סכום"),
    ("admin.invoice", "Monthly charge", "חיוב חודשי"),
    ("admin.open_in_ls", "Open / refund", "פתיחה / החזר"),
    ("admin.no_log", "No admin actions yet.", "עדיין אין פעולות ניהול."),
    ("err.signup_google", "Please sign up with Google", "אנא הירשמו עם Google"),
    ("err.suspended", "This account is suspended", "החשבון הזה מושעה"),
    ("err.payments_closed", "Payments are not open yet", "הרכישה עדיין לא פתוחה"),
    ("err.admins_only", "Admins only", "למנהלים בלבד"),
    ("auth.or", "or", "או"),
    ("auth.google_wait", "Finish signing in in your browser — this screen continues by itself.", "סיימו את ההתחברות בדפדפן — המסך הזה ימשיך לבד."),
    ("auth.google_failed", "Signing in with Google didn't work", "ההתחברות עם Google לא הצליחה"),
    ("avatar.title", "Profile picture", "תמונת פרופיל"),
    ("avatar.sub", "Pick a character, take a photo or upload one from your device.", "בחרו דמות, צלמו תמונה או העלו תמונה מהמכשיר."),
    ("avatar.camera", "Take a photo", "צילום תמונה"),
    ("avatar.upload", "Upload from device", "העלאה מהמכשיר"),
    ("avatar.save", "Save", "שמירה"),
    ("avatar.skip", "Skip for now", "דלגו בינתיים"),
    ("avatar.cancel", "Cancel", "ביטול"),
    ("avatar.bad_image", "That file couldn't be opened as a picture.", "לא ניתן לפתוח את הקובץ הזה כתמונה."),
    ("profile.days", "{} days", "{} ימים"),
    ("profile.league", "League", "ליגה"),
    ("profile.share", "Share your progress", "שתפו את ההתקדמות"),
    ("profile.copied", "Copied, paste it anywhere", "הועתק, אפשר להדביק בכל מקום"),
    ("profile.share_text", "I'm learning to code on Code-Iot: {} XP and a {}-day streak!", "אני לומד/ת לתכנת ב-Code-Iot: {} XP ורצף של {} ימים!"),
    ("profile.more", "More stats", "עוד נתונים"),
    ("tier.bronze", "Bronze", "ארד"),
    ("tier.silver", "Silver", "כסף"),
    ("tier.gold", "Gold", "זהב"),
    ("tier.diamond", "Diamond", "יהלום"),
    // playground
    ("pg.title", "Playground", "מגרש משחקים"),
    ("pg.reset", "Reset", "איפוס"),
    ("pg.run", "Run", "הרצה"),
    ("pg.result", "Result", "תוצאה"),
    ("pg.tip", "Tip: code runs in a sandbox right in your browser.", "טיפ: הקוד רץ בסביבה מבודדת ישירות בדפדפן."),
    ("pg.code", "Code", "קוד"),
    ("pg.input", "Input", "קלט"),
    ("pg.output", "Output", "פלט"),
    ("pg.running", "Compiling and running…", "מקמפל ומריץ…"),
    ("pg.input_hint", "Text your program reads: input(), std::cin, stdin or Serial. One answer per line.", "טקסט שהתוכנית קוראת: input()‏, std::cin‏, stdin או Serial. תשובה אחת בכל שורה."),
    ("pg.no_output", "(no output)", "(אין פלט)"),
    ("pg.compile_error", "Compile error", "שגיאת קומפילציה"),
    ("pg.runtime_error", "The program exited with code {}", "התוכנית הסתיימה עם קוד {}"),
    ("pg.press_run", "Press Run to see the output here.", "לחצו על הרצה כדי לראות כאן את הפלט."),
    ("pg.tip_python", "Python 3 runs on a remote sandbox. Anything your program reads with input() comes from the Input tab, one line per call.", "Python 3 רץ בסביבה מבודדת מרוחקת. מה שהתוכנית קוראת עם input() מגיע מלשונית הקלט, שורה אחת לכל קריאה."),
    ("pg.tip_js", "JavaScript runs in a sandbox in your browser; console.log prints here.", "JavaScript רץ בסביבה מבודדת בדפדפן; console.log מדפיס לכאן."),
    ("pg.tip_sql", "SQL runs on a fresh SQLite database in your browser. Every run starts empty.", "SQL רץ על מסד SQLite חדש בדפדפן. כל הרצה מתחילה ממסד ריק."),
    ("pg.tip_server", "Compiled and run on Compiler Explorer (godbolt.org), so it needs an internet connection.", "מקומפל ורץ ב-Compiler Explorer ‏(godbolt.org), ולכן צריך חיבור לאינטרנט."),
    ("pg.tip_arduino", "Runs on a simulated Uno: delay() is instant, pin changes are printed, a button on input pins is pressed every 3 s, and Input is what you type into the Serial Monitor. It stops after 20 s of board time.", "רץ על Uno מדומה: delay() מיידי, שינויים בפינים מודפסים, כפתור בפיני קלט נלחץ כל 3 שניות, והקלט הוא מה שמקלידים ב-Serial Monitor. ההרצה נעצרת אחרי 20 שניות של זמן לוח."),
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
    ("err.avatar_unknown", "Unknown avatar", "דמות לא מוכרת"),
    ("err.avatar_type", "The picture must be", "התמונה חייבת להיות JPEG,‏ PNG או WebP"),
    ("err.avatar_large", "The picture is too large", "התמונה גדולה מדי"),
    ("err.avatar_data", "The picture data is not valid", "נתוני התמונה לא תקינים"),
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
