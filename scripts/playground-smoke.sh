#!/usr/bin/env bash
# Smoke test for the playground: starts the built server, runs a C++, Rust and
# Arduino program through /api/run, and checks the browser runtimes exist.
set -u
export ADDR=127.0.0.1:3999 DATABASE_PATH=/tmp/smoke.db WEB_DIST=web/dist
./target/release/codesprout-server & pid=$!
sleep 5
echo "--- database: ${TURSO_DATABASE_URL:+Turso}${TURSO_DATABASE_URL:-local file}"
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
import json, os, urllib.request
B = "http://127.0.0.1:3999"
def call(method, path, body=None, token=None):
    h = {"Content-Type": "application/json"}
    if token: h["Authorization"] = "Bearer " + token
    req = urllib.request.Request(B + path, json.dumps(body).encode() if body is not None else None, h, method=method)
    try:
        return json.load(urllib.request.urlopen(req, timeout=20))
    except urllib.error.HTTPError as e:
        return {"http_error": e.code, "body": e.read().decode()[:200]}
import secrets
name = "smoke_" + secrets.token_hex(4)
r = call("POST", "/api/auth/signup", {"username": name, "password": "secret123"})
if "token" not in r:
    print("SIGNUP FAILED:", r); raise SystemExit(0)
tok = r["token"]; print("signup avatar:", r["profile"].get("avatar"))
print("preset:", call("PUT", "/api/me/avatar", {"avatar": "preset:a2"}, tok).get("avatar"))
print("me:", call("GET", "/api/me", token=tok).get("avatar"))
print("upload:", call("PUT", "/api/me/avatar", {"avatar": "data:image/jpeg;base64,/9j/4AAQ"}, tok).get("avatar"))
print("bad:", call("PUT", "/api/me/avatar", {"avatar": "data:text/html;base64,PGg+"}, tok))
print("clear:", call("PUT", "/api/me/avatar", {"avatar": None}, tok).get("avatar"))
login = call("POST", "/api/auth/login", {"username": name, "password": "secret123"})
print("login again:", "token" in login, "| completions saved:", call("POST", "/api/lessons/py-1-1/complete", {"correct": 3, "tz_offset_minutes": 120}, tok).get("xp_gained"))
print("profile after lesson: xp", call("GET", "/api/me", token=tok).get("xp"))

print("--- access")
import hashlib, hmac
lessons = call("GET", "/api/courses/python")["sections"]
locked_guest = lessons[2]["lessons"][0]["id"]
print("guest, topic 3:", call("GET", f"/api/lessons/{locked_guest}"))
print("guest, topic 1:", "steps" in call("GET", "/api/lessons/" + lessons[0]["lessons"][0]["id"]))
print("user, topic 2:", "steps" in call("GET", "/api/lessons/" + lessons[1]["lessons"][0]["id"], token=tok))
print("user, topic 3:", call("GET", f"/api/lessons/{locked_guest}", token=tok))
print("user sees locks:", [l["locked"] for l in call("GET", "/api/courses/python", token=tok)["sections"][1:3] for l in l["lessons"][:1]])
print("admin api as a user:", call("GET", "/api/admin/overview", token=tok))
print("payments config:", call("GET", "/api/payments/config"))
me = call("GET", "/api/me", token=tok)
secret = os.environ.get("LEMONSQUEEZY_WEBHOOK_SECRET", "")
if secret:
    def post_hook(raw, sig):
        req = urllib.request.Request(B + "/api/payments/webhook", raw.encode(),
                                     {"Content-Type": "application/json", "X-Signature": sig})
        try:
            return urllib.request.urlopen(req, timeout=20).status
        except urllib.error.HTTPError as e:
            return e.code
    body = json.dumps({"meta": {"event_name": "order_created", "test_mode": True,
                                "custom_data": {"user_id": str(me["id"]), "course_id": "python"}},
                       "data": {"type": "orders", "id": "smoke-" + name, "attributes": {"status": "paid", "total": 2500,
                                "currency": "USD", "user_email": "smoke@example.com",
                                "first_order_item": {"variant_id": os.environ.get("LEMONSQUEEZY_VARIANT_LIFETIME", "")}}}})
    good = hmac.new(secret.encode(), body.encode(), hashlib.sha256).hexdigest()
    print("webhook, bad signature:", post_hook(body, "00" * 32))
    print("webhook, signed:", post_hook(body, good))
    print("python unlocked after paying:", call("GET", "/api/courses/python", token=tok).get("unlocked"))
    print("user, topic 3 after paying:", "steps" in call("GET", f"/api/lessons/{locked_guest}", token=tok))
    print("plans:", call("GET", "/api/me", token=tok).get("plans"))
    print("rust still locked:", call("GET", "/api/courses/rust", token=tok).get("unlocked") is False)
print("--- google")
print("providers:", call("GET", "/api/auth/providers"))
class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *a, **k): return None
opener = urllib.request.build_opener(NoRedirect)
def location(path):
    try:
        opener.open(B + path, timeout=20); return "(no redirect)"
    except urllib.error.HTTPError as e:
        return f"{e.code} {e.headers.get('Location', '')[:160]}"
print("start:", location("/api/auth/google/start?poll=" + "a" * 32))
print("callback with a bad state:", location("/api/auth/google/callback?state=nope&code=x"))
print("poll:", call("GET", "/api/auth/google/poll?id=" + "a" * 32))
PY
kill $pid
for u in https://cdn.jsdelivr.net/pyodide/v0.26.4/full/pyodide.js \
         https://cdn.jsdelivr.net/npm/sql.js@1.10.3/dist/sql-wasm.js \
         https://cdn.jsdelivr.net/npm/sql.js@1.10.3/dist/sql-wasm.wasm; do
  echo "--- $(curl -s -o /dev/null -w '%{http_code}' "$u") $u"
done
exit 0
