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
cat > /tmp/bad.cpp <<'B'
int main() { return x; }
B
run cpp /tmp/a.cpp "Code-Iot"
run rust /tmp/a.rs "Code-Iot"
run arduino /tmp/a.ino ""
run cpp /tmp/bad.cpp ""
kill $pid
for u in https://cdn.jsdelivr.net/pyodide/v0.26.4/full/pyodide.js \
         https://cdn.jsdelivr.net/npm/sql.js@1.10.3/dist/sql-wasm.js \
         https://cdn.jsdelivr.net/npm/sql.js@1.10.3/dist/sql-wasm.wasm; do
  echo "--- $(curl -s -o /dev/null -w '%{http_code}' "$u") $u"
done
exit 0
