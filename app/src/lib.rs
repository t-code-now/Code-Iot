//! The Android app is a native window around the Code-Iot web app
//! (built from `web/` into `web/dist`). It talks to the hosted server
//! whose address is baked in at build time (`CODESPROUT_API`).

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Opens links (e.g. "Continue with Google") in the phone's browser.
        .plugin(tauri_plugin_opener::init())
        // In-app "Sign in with Google" (no browser).
        .plugin(tauri_plugin_google_signin::init())
        .run(tauri::generate_context!())
        .expect("failed to start Code-Iot");
}
