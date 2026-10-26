//! `POST /api/run`: compiles and runs playground code for languages a
//! browser can't run reliably by itself (Python, C++, Rust, and Arduino sketches through the
//! simulator). The code is executed by Compiler Explorer (godbolt.org) in its
//! own sandbox; nothing runs on this server.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::sketch::arduino_program;

const GODBOLT: &str = "https://godbolt.org/api";
const MAX_CODE: usize = 64 * 1024;
const MAX_STDIN: usize = 16 * 1024;

#[derive(Deserialize)]
pub struct RunRequest {
    pub language: String,
    pub code: String,
    #[serde(default)]
    pub stdin: String,
}

#[derive(Serialize)]
pub struct RunResponse {
    /// "ok", "compile_error", "runtime_error" or "unavailable".
    pub status: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i64>,
}

fn reply(status: &str, stdout: String, stderr: String, exit_code: Option<i64>) -> Json<RunResponse> {
    Json(RunResponse { status: status.into(), stdout, stderr, exit_code })
}

fn client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(45))
            .user_agent("Code-Iot playground (https://github.com/t-code-now/Code-Iot)")
            .build()
            .expect("http client")
    })
}

/// Picks the newest stable compiler of a family from Compiler Explorer's list,
/// cached for a few hours. Falls back to a known id if the list can't be read.
async fn compiler_id(lang: &str) -> String {
    static CACHE: Mutex<Vec<(String, String, Instant)>> = Mutex::new(Vec::new());
    if let Some((_, id, _)) = CACHE
        .lock()
        .unwrap()
        .iter()
        .find(|(l, _, at)| l == lang && at.elapsed() < Duration::from_secs(6 * 3600))
    {
        return id.clone();
    }
    let fallback = match lang {
        "rust" => "r1820",
        "python" => "python312",
        _ => "g132",
    };
    let url = format!("{GODBOLT}/compilers/{lang}?fields=id,name,semver,instructionSet,supportsExecute");
    let list: Vec<Value> = match client().get(&url).header("Accept", "application/json").send().await {
        Ok(r) => r.json().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let wanted = |c: &&Value| {
        let name = c["name"].as_str().unwrap_or("").to_lowercase();
        let semver = c["semver"].as_str().unwrap_or("");
        let executes = c["supportsExecute"].as_bool().unwrap_or(true);
        let x86 = lang != "c++" || c["instructionSet"].as_str().map_or(true, |s| s == "amd64");
        let stable = semver.chars().next().is_some_and(|ch| ch.is_ascii_digit())
            && !semver.contains("trunk")
            && !semver.contains("nightly")
            && !semver.contains("beta");
        let family = match lang {
            "rust" => name.starts_with("rustc ") && !name.contains("gcc") && !name.contains("mrustc"),
            "python" => name.starts_with("python 3") && !name.contains("pypy"),
            _ => name.starts_with("x86-64 gcc ") && !name.contains("assert") && !name.contains("("),
        };
        executes && x86 && stable && family
    };
    let version = |c: &Value| -> Vec<u32> {
        c["semver"].as_str().unwrap_or("").split('.').map(|p| p.parse().unwrap_or(0)).collect()
    };
    let id = list
        .iter()
        .filter(wanted)
        .max_by_key(|c| version(c))
        .and_then(|c| c["id"].as_str())
        .unwrap_or(fallback)
        .to_string();
    let mut cache = CACHE.lock().unwrap();
    cache.retain(|(l, _, _)| l != lang);
    cache.push((lang.to_string(), id.clone(), Instant::now()));
    id
}

fn join(lines: &Value) -> String {
    lines
        .as_array()
        .map(|a| a.iter().filter_map(|l| l["text"].as_str()).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

/// Removes Compiler Explorer's ANSI colour codes and temp paths from messages.
fn tidy(text: String) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for n in chars.by_ref() {
                if n.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out.replace("<source>", "main")
        .replace("/app/example.cpp", "main")
        .replace("/app/example.py", "main.py")
        .replace("/app/output.s", "main.py")
        .replace("example.py", "main.py")
}

pub async fn run(Json(req): Json<RunRequest>) -> Result<Json<RunResponse>, (StatusCode, Json<Value>)> {
    if req.code.len() > MAX_CODE || req.stdin.len() > MAX_STDIN {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, Json(json!({ "error": "Code is too long" }))));
    }
    let (lang, source, flags) = match req.language.as_str() {
        "cpp" => ("c++", req.code.clone(), "-std=c++20 -O1"),
        "arduino" => ("c++", arduino_program(&req.code), "-std=gnu++17 -O1 -w"),
        "rust" => ("rust", req.code.clone(), "--edition 2021 -C opt-level=1"),
        "python" => ("python", req.code.clone(), ""),
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Unknown language" })))),
    };
    let id = compiler_id(lang).await;
    let body = json!({
        "source": source,
        "compiler": id,
        "lang": lang,
        "options": {
            "userArguments": flags,
            "executeParameters": { "args": [], "stdin": req.stdin },
            "compilerOptions": { "executorRequest": true },
            "filters": { "execute": true },
            "tools": [],
            "libraries": []
        },
        "allowStoreCodeDebug": false
    });
    let resp = client()
        .post(format!("{GODBOLT}/compiler/{id}/compile"))
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await;
    let value: Value = match resp {
        Ok(r) if r.status().is_success() => match r.json().await {
            Ok(v) => v,
            Err(e) => return Ok(reply("unavailable", String::new(), format!("Unexpected reply from the compiler service ({e})"), None)),
        },
        Ok(r) => return Ok(reply("unavailable", String::new(), format!("The compiler service is busy (HTTP {}). Try again in a moment.", r.status()), None)),
        Err(e) => return Ok(reply("unavailable", String::new(), format!("Can't reach the compiler service ({e})"), None)),
    };

    // Execution replies carry the build in `buildResult`; some versions nest
    // the run in `execResult` instead.
    let exec = if value.get("execResult").is_some() { &value["execResult"] } else { &value };
    let build = if value.get("buildResult").is_some() { &value["buildResult"] } else { exec.get("buildResult").unwrap_or(&Value::Null) };
    let build_code = build["code"].as_i64().unwrap_or(0);
    if build_code != 0 || exec["didExecute"].as_bool() == Some(false) {
        let mut msg = tidy(join(&build["stderr"]));
        if msg.trim().is_empty() {
            msg = tidy(join(&build["stdout"]));
        }
        if msg.trim().is_empty() {
            msg = tidy(join(&exec["stderr"]));
        }
        return Ok(reply("compile_error", String::new(), msg, Some(build_code)));
    }
    let code = exec["code"].as_i64();
    let mut stderr = tidy(join(&exec["stderr"]));
    if exec["timedOut"].as_bool() == Some(true) {
        stderr.push_str("\n[stopped: the program ran for too long]");
    }
    let status = if code == Some(0) { "ok" } else { "runtime_error" };
    Ok(reply(status, join(&exec["stdout"]), stderr, code))
}
