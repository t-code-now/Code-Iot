#!/usr/bin/env bash
# Smoke test for the playground: starts the built server, runs a C++, Rust and
# Arduino program through /api/run, and checks the browser runtimes exist.
set -u
export ADDR=127.0.0.1:3999 DATABASE_PATH=/tmp/smoke.db WEB_DIST=web/dist
./target/release/codesprout-server & pid=$!
sleep 3
run() {
  echo "--- /api/run $1"
  python3 - "$1" "$2" "$3" <<'PY'
import json, sys, urllib.request
lang, code, stdin = sys.argv[1], open(sys.argv[2]).read(), sys.argv[3]
req = urllib.request.Request("http://127.0.0.1:3999/api/run", json.dumps({"language": lang, "code": code, "stdin": stdin}).encode(),
                             {"Content-Type": "application/json"})
r = json.load(urllib.request.urlopen(req, timeout=90))
print("status:", r["status"], "exit:", r["exit_code"])
print("stdout:", r["stdout"][:600])
print("stderr:", r["stderr"][:600])
PY
}
cat > /tmp/a.cpp <<'C'
#include <iostream>
#include <string>
int main() { std::string n; std::getline(std::cin, n); std::cout << "Hello, " << n << "!\n"; }
C
cat > /tmp/a.rs <<'R'
use std::io::Read;
fn main() { let mut s = String::new(); std::io::stdin().read_to_string(&mut s).unwrap(); println!("Hello, {}!", s.trim()); }
R
cat > /tmp/a.ino <<'I'
void setup() { pinMode(13, OUTPUT); Serial.begin(9600); Serial.println("hi from the sim"); }
void loop() { digitalWrite(13, HIGH); delay(1000); digitalWrite(13, LOW); delay(1000); }
I
cat > /tmp/a.py <<'P'
name = input("What's your name? ")
print(f"Hello, {name}!")
for i in range(1, 4):
    print(i, "squared is", i * i)
P
cat > /tmp/bad.py <<'P'
print(1 / 0)
P
cat > /tmp/bad.cpp <<'B'
int main() { return x; }
B
run cpp /tmp/a.cpp "Code-Iot"
run rust /tmp/a.rs "Code-Iot"
run arduino /tmp/a.ino ""
run cpp /tmp/bad.cpp ""
run python /tmp/a.py "Code-Iot"
run python /tmp/bad.py ""
echo "--- avatar"
python3 - <<'PY'
import json, urllib.request
B = "http://127.0.0.1:3999"
def call(method, path, body=None, token=None):
    h = {"Content-Type": "application/json"}
    if token: h["Authorization"] = "Bearer " + token
    req = urllib.request.Request(B + path, json.dumps(body).encode() if body is not None else None, h, method=method)
    try:
        return json.load(urllib.request.urlopen(req, timeout=20))
    except urllib.error.HTTPError as e:
        return {"http_error": e.code, "body": e.read().decode()[:200]}
r = call("POST", "/api/auth/signup", {"username": "avatar_smoke", "password": "secret123"})
tok = r["token"]; print("signup avatar:", r["profile"].get("avatar"))
print("preset:", call("PUT", "/api/me/avatar", {"avatar": "preset:a2"}, tok).get("avatar"))
print("me:", call("GET", "/api/me", token=tok).get("avatar"))
print("upload:", call("PUT", "/api/me/avatar", {"avatar": "data:image/jpeg;base64,/9j/4AAQ"}, tok).get("avatar"))
print("bad:", call("PUT", "/api/me/avatar", {"avatar": "data:text/html;base64,PGg+"}, tok))
print("clear:", call("PUT", "/api/me/avatar", {"avatar": None}, tok).get("avatar"))
PY
kill $pid
for u in https://cdn.jsdelivr.net/pyodide/v0.26.4/full/pyodide.js \
         https://cdn.jsdelivr.net/npm/sql.js@1.10.3/dist/sql-wasm.js \
         https://cdn.jsdelivr.net/npm/sql.js@1.10.3/dist/sql-wasm.wasm; do
  echo "--- $(curl -s -o /dev/null -w '%{http_code}' "$u") $u"
done
exit 0
