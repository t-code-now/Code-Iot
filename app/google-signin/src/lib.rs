//! In-app "Sign in with Google" for the Android app, using Android's
//! Credential Manager: Google's account sheet opens over the app and returns
//! an ID token, which the Code-Iot server checks with Google. No browser.

use serde::{Deserialize, Serialize};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInRequest {
    /// The server's Google *web* client id (the ID token's audience).
    pub client_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInResponse {
    pub id_token: String,
}

#[cfg(target_os = "android")]
struct GoogleSignIn<R: Runtime>(tauri::plugin::PluginHandle<R>);

#[tauri::command]
async fn sign_in<R: Runtime>(app: tauri::AppHandle<R>, client_id: String) -> Result<SignInResponse, String> {
    #[cfg(target_os = "android")]
    {
        let handle = &app.state::<GoogleSignIn<R>>().0;
        return handle
            .run_mobile_plugin::<SignInResponse>("signIn", SignInRequest { client_id })
            .map_err(|e| e.to_string());
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, client_id);
        Err("unsupported".into())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("google-signin")
        .invoke_handler(tauri::generate_handler![sign_in])
        .setup(|_app, _api| {
            #[cfg(target_os = "android")]
            {
                let handle = _api.register_android_plugin("com.tcodenow.googlesignin", "GoogleSignInPlugin")?;
                _app.manage(GoogleSignIn(handle));
            }
            Ok(())
        })
        .build()
}
