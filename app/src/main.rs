// Desktop entry point (the Android build uses `run` from lib.rs).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    codesprout_app_lib::run()
}
