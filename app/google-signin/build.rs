const COMMANDS: &[&str] = &["sign_in"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).android_path("android").build();
}
