//! The Android app is a native window around the CodeSprout web app
//! (built from `web/` into `web/dist`). It talks to the hosted server
//! whose address is baked in at build time (`CODESPROUT_API`).

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to start CodeSprout");
}
