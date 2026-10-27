#!/usr/bin/env python3
"""End-to-end test of purchases, refunds and cancellations.

Starts the built server against a local SQLite file and a fake Lemon Squeezy
API that records every request, then walks through: phone required before
paying, checkout, signed webhooks, partial and full refunds (with limits),
access closing after a full refund, cancelling a subscription, pausing sales,
and that only the admin may touch money. Exits 1 if anything is wrong.
"""
import hashlib, hmac, json, os, sqlite3, subprocess, sys, threading, time, urllib.error, urllib.request
from http.server import BaseHTTPRequestHandler, HTTPServer

API_KEY, SECRET = "test-key", "test-hook-secret"
VAR_MONTHLY, VAR_LIFETIME = "111", "222"
BASE = "http://127.0.0.1:3998"
DB = "/tmp/payments-test.db"
calls = []   # requests seen by the fake Lemon Squeezy
refunded = {}  # order/invoice id -> cents refunded so far
failures = []


def check(name, ok, detail=""):
    print(("PASS " if ok else "FAIL ") + name + (f"  [{detail}]" if detail and not ok else ""))
    if not ok:
        failures.append(name)


class FakeLemonSqueezy(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def reply(self, code, body):
        raw = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/vnd.api+json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def handle_any(self):
        length = int(self.headers.get("Content-Length") or 0)
        body = json.loads(self.rfile.read(length) or b"null")
        calls.append({"method": self.command, "path": self.path, "body": body, "auth": self.headers.get("Authorization")})
        if self.headers.get("Authorization") != f"Bearer {API_KEY}":
            return self.reply(401, {"errors": [{"detail": "bad key"}]})
        parts = self.path.strip("/").split("/")  # v1, kind, id, action
        if self.command == "POST" and parts[1:] == ["checkouts"]:
            return self.reply(201, {"data": {"attributes": {"url": "https://checkout.fake/abc"}}})
        if self.command == "POST" and len(parts) == 4 and parts[3] == "refund":
            kind, rid = parts[1], parts[2]
            amount = body["data"]["attributes"].get("amount")
            totals = {"5001": 2500, "9001": 500}
            done = refunded.get(rid, 0) + amount
            if done > totals.get(rid, 0):
                return self.reply(422, {"errors": [{"detail": "Refund exceeds the order total"}]})
            refunded[rid] = done
            status = "refunded" if done == totals[rid] else "partial_refund"
            return self.reply(200, {"data": {"type": kind, "id": rid, "attributes": {"status": status, "refunded_amount": done}}})
        if self.command == "DELETE" and parts[1] == "subscriptions":
            return self.reply(200, {"data": {"type": "subscriptions", "id": parts[2],
                                              "attributes": {"status": "cancelled", "ends_at": "2031-01-01T00:00:00.000000Z"}}})
        return self.reply(404, {"errors": [{"detail": "unknown"}]})

    do_GET = do_POST = do_DELETE = do_PATCH = handle_any


def call(method, path, body=None, token=None):
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = "Bearer " + token
    req = urllib.request.Request(BASE + path, json.dumps(body).encode() if body is not None else None, headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return r.status, json.loads(r.read() or b"null")
    except urllib.error.HTTPError as e:
        try:
            return e.code, json.loads(e.read() or b"null")
        except Exception:
            return e.code, None


def hook(event):
    raw = json.dumps(event).encode()
    sig = hmac.new(SECRET.encode(), raw, hashlib.sha256).hexdigest()
    req = urllib.request.Request(BASE + "/api/payments/webhook", raw, {"Content-Type": "application/json", "X-Signature": sig})
    try:
        return urllib.request.urlopen(req, timeout=20).status
    except urllib.error.HTTPError as e:
        return e.code


def main():
    fake = HTTPServer(("127.0.0.1", 4010), FakeLemonSqueezy)
    threading.Thread(target=fake.serve_forever, daemon=True).start()
    if os.path.exists(DB):
        os.remove(DB)
    env = dict(os.environ, ADDR="127.0.0.1:3998", DATABASE_PATH=DB, WEB_DIST="web/dist", ALLOW_PASSWORD_SIGNUP="1",
               LEMONSQUEEZY_API_KEY=API_KEY, LEMONSQUEEZY_STORE_ID="1", LEMONSQUEEZY_VARIANT_MONTHLY=VAR_MONTHLY,
               LEMONSQUEEZY_VARIANT_LIFETIME=VAR_LIFETIME, LEMONSQUEEZY_WEBHOOK_SECRET=SECRET,
               LEMONSQUEEZY_API_BASE="http://127.0.0.1:4010/v1", ADMIN_EMAILS="admin@example.com",
               TURSO_DATABASE_URL="", TURSO_AUTH_TOKEN="")
    server = subprocess.Popen(["./target/release/codesprout-server"], env=env)
    try:
        for _ in range(50):
            try:
                urllib.request.urlopen(BASE + "/api/health", timeout=2)
                break
            except Exception:
                time.sleep(0.2)
        run()
    finally:
        server.kill()
        fake.shutdown()
    print(f"\n{len(failures)} failure(s)" + (": " + ", ".join(failures) if failures else ""))
    sys.exit(1 if failures else 0)


def run():
    _, buyer = call("POST", "/api/auth/signup", {"username": "buyer", "password": "secret123"})
    _, admin = call("POST", "/api/auth/signup", {"username": "boss", "password": "secret123"})
    tok, atok = buyer["token"], admin["token"]
    buyer_id, admin_id = buyer["profile"]["id"], admin["profile"]["id"]
    # Make "boss" the admin: a verified Google e-mail on the admin list.
    con = sqlite3.connect(DB)
    con.execute("INSERT INTO google_accounts (sub, user_id, email) VALUES ('g-admin', ?, 'admin@example.com')", (admin_id,))
    con.commit()
    con.close()

    check("admin flag", call("GET", "/api/me", token=atok)[1].get("is_admin") is True)
    check("buyer is not admin", call("GET", "/api/me", token=tok)[1].get("is_admin") is False)

    # --- phone before paying
    code, body = call("POST", "/api/payments/checkout", {"course_id": "python", "plan": "lifetime"}, tok)
    check("checkout needs a phone", code == 400 and "phone" in (body or {}).get("error", ""), str((code, body)))
    code, body = call("PUT", "/api/me/contact", {"country": "IL", "phone": "050-123-4567"}, tok)
    check("phone saved in international form", code == 200 and body.get("phone") == "+972501234567" and body.get("country") == "IL", str(body))
    code, body = call("PUT", "/api/me/contact", {"country": "IL", "phone": "12"}, tok)
    check("bad phone rejected", code == 400, str((code, body)))

    code, body = call("POST", "/api/payments/checkout", {"course_id": "python", "plan": "lifetime"}, tok)
    check("checkout url", code == 200 and body.get("url") == "https://checkout.fake/abc", str((code, body)))
    sent = calls[-1]["body"]["data"]
    check("checkout sends the lifetime variant", sent["relationships"]["variant"]["data"]["id"] == VAR_LIFETIME)
    check("checkout carries user and course", sent["attributes"]["checkout_data"]["custom"] == {"user_id": str(buyer_id), "course_id": "python", "plan": "lifetime"})
    check("checkout prefills the country", sent["attributes"]["checkout_data"].get("billing_address") == {"country": "IL"})

    # --- the payment arrives
    code, body = call("GET", "/api/courses/?lang=he", token=tok)
    check("unknown API path answers JSON 404", code == 404 and isinstance(body, dict) and "error" in body, str((code, body)))
    check("python locked before paying", call("GET", "/api/courses/python", token=tok)[1]["unlocked"] is False)
    order = {"meta": {"event_name": "order_created", "custom_data": {"user_id": str(buyer_id), "course_id": "python"}},
             "data": {"type": "orders", "id": "5001", "attributes": {"status": "paid", "total": 2500, "currency": "USD",
                      "user_email": "buyer@example.com", "user_name": "Buyer One", "test_mode": True,
                      "first_order_item": {"variant_id": int(VAR_LIFETIME)}, "urls": {"receipt": "https://r/1"},
                      "created_at": "2026-10-27T08:00:00.000000Z"}}}
    raw = json.dumps(order).encode()
    bad = urllib.request.Request(BASE + "/api/payments/webhook", raw, {"Content-Type": "application/json", "X-Signature": "00" * 32})
    try:
        urllib.request.urlopen(bad, timeout=10)
        check("forged webhook rejected", False)
    except urllib.error.HTTPError as e:
        check("forged webhook rejected", e.code == 401)
    check("python still locked after a forged webhook", call("GET", "/api/courses/python", token=tok)[1]["unlocked"] is False)
    check("signed webhook accepted", hook(order) == 200)
    check("python opens after paying", call("GET", "/api/courses/python", token=tok)[1]["unlocked"] is True)
    check("rust stays locked", call("GET", "/api/courses/rust", token=tok)[1]["unlocked"] is False)
    check("same webhook twice is harmless", hook(order) == 200)

    _, pays = call("GET", "/api/admin/payments", token=atok)
    pay = next(p for p in pays if p["provider_ref"] == "ord_5001")
    check("one record per order", sum(1 for p in pays if p["provider_ref"] == "ord_5001") == 1)
    check("payment details", pay["amount_cents"] == 2500 and pay["refundable_cents"] == 2500 and pay["test_mode"]
          and pay["customer_name"] == "Buyer One" and pay["phone"] == "+972501234567" and pay["country"] == "IL", str(pay))

    # --- only the admin touches money
    code, _ = call("POST", "/api/admin/refund", {"payment_id": pay["id"], "amount_cents": 100}, tok)
    check("buyer can't refund", code == 403)
    code, _ = call("POST", "/api/admin/refund", {"payment_id": pay["id"], "amount_cents": 100})
    check("guest can't refund", code == 401)

    # --- refunds with limits
    before = len(calls)
    code, body = call("POST", "/api/admin/refund", {"payment_id": pay["id"], "amount_cents": 3000}, atok)
    check("refund above the payment rejected", code == 400 and len(calls) == before, str((code, body)))
    code, body = call("POST", "/api/admin/refund", {"payment_id": pay["id"], "amount_cents": 0}, atok)
    check("zero refund rejected", code == 400 and len(calls) == before)
    code, body = call("POST", "/api/admin/refund", {"payment_id": pay["id"], "amount_cents": 1000}, atok)
    last = calls[-1]
    check("partial refund", code == 200 and body.get("status") == "partial_refund" and body.get("refunded_cents") == 1000, str((code, body)))
    check("refund request sent correctly", last["method"] == "POST" and last["path"] == "/v1/orders/5001/refund"
          and last["body"]["data"]["attributes"]["amount"] == 1000 and last["body"]["data"]["type"] == "orders", str(last))
    _, pays = call("GET", "/api/admin/payments", token=atok)
    pay = next(p for p in pays if p["provider_ref"] == "ord_5001")
    check("refundable left", pay["refundable_cents"] == 1500 and pay["refunded_cents"] == 1000, str(pay))
    check("partial refund keeps access", call("GET", "/api/courses/python", token=tok)[1]["unlocked"] is True)
    code, body = call("POST", "/api/admin/refund", {"payment_id": pay["id"], "amount_cents": 1501}, atok)
    check("can't refund more than what's left", code == 400, str((code, body)))
    code, body = call("POST", "/api/admin/refund", {"payment_id": pay["id"], "amount_cents": 1500}, atok)
    check("rest refunded", code == 200 and body.get("status") == "refunded", str((code, body)))
    check("full refund closes the course", call("GET", "/api/courses/python", token=tok)[1]["unlocked"] is False)
    code, body = call("POST", "/api/admin/refund", {"payment_id": pay["id"], "amount_cents": 100}, atok)
    check("nothing left to refund", code == 400)
    check("repeated 'paid' message after a refund", hook(order) == 200)
    check("a late 'paid' message doesn't reopen a refunded course", call("GET", "/api/courses/python", token=tok)[1]["unlocked"] is False)
    check("Lemon Squeezy got exactly two refund calls", sum(1 for c in calls if c["path"].endswith("/refund")) == 2)

    # --- subscription: invoice refund and cancellation
    sub = {"meta": {"event_name": "subscription_created", "custom_data": {"user_id": str(buyer_id), "course_id": "rust"}},
           "data": {"type": "subscriptions", "id": "7001", "attributes": {"variant_id": int(VAR_MONTHLY), "status": "active",
                    "card_brand": "visa", "card_last_four": "4242", "renews_at": "2031-01-01T00:00:00.000000Z",
                    "user_email": "buyer@example.com", "urls": {"customer_portal": "https://portal"}}}}
    inv = {"meta": {"event_name": "subscription_payment_success", "custom_data": {"user_id": str(buyer_id), "course_id": "rust"}},
           "data": {"type": "subscription-invoices", "id": "9001", "attributes": {"subscription_id": 7001, "status": "paid",
                    "total": 500, "currency": "USD", "card_brand": "visa", "card_last_four": "4242"}}}
    check("subscription webhook", hook(sub) == 200 and hook(inv) == 200)
    check("rust opens with a subscription", call("GET", "/api/courses/rust", token=tok)[1]["unlocked"] is True)
    _, pays = call("GET", "/api/admin/payments", token=atok)
    s = next(p for p in pays if p["provider_ref"] == "sub_7001")
    i = next(p for p in pays if p["provider_ref"] == "inv_9001")
    check("subscription can be cancelled, not refunded", s["can_cancel"] and s["refundable_cents"] == 0)
    check("invoice can be refunded", i["refundable_cents"] == 500 and i["method"] == "Visa •••• 4242", str(i))
    code, body = call("POST", "/api/admin/refund", {"payment_id": s["id"], "amount_cents": 100}, atok)
    check("refunding the subscription record itself is refused", code == 400)
    code, body = call("POST", "/api/admin/refund", {"payment_id": i["id"], "amount_cents": 200}, atok)
    check("invoice partial refund", code == 200 and calls[-1]["path"] == "/v1/subscription-invoices/9001/refund"
          and calls[-1]["body"]["data"]["type"] == "subscription-invoices", str((code, body, calls[-1])))
    check("no unused-part refund before cancelling", i.get("prorata_cents") == 0, str(i.get("prorata_cents")))
    _, mine = call("GET", "/api/me/purchases", token=tok)
    ms = next((m for m in mine if m["id"] == s["id"]), None)
    check("buyer sees own subscription with its charges", ms is not None and ms["amount_cents"] == 500
          and ms["refunded_cents"] == 200 and ms["can_cancel"] is True and not any(m["id"] == i["id"] for m in mine), str(mine))
    _, other = call("GET", "/api/me/purchases", token=atok)
    check("another user doesn't see it", not any(m["id"] == s["id"] for m in other), str(other))
    code, _ = call("POST", "/api/me/cancel", {"payment_id": s["id"]})
    check("guest can't cancel", code == 401, str(code))
    n_calls = len(calls)
    code, _ = call("POST", "/api/me/cancel", {"payment_id": s["id"]}, atok)
    check("another user can't cancel someone else's subscription", code == 404 and len(calls) == n_calls, str(code))
    code, body = call("POST", "/api/me/cancel", {"payment_id": s["id"]}, tok)
    check("cancel subscription", code == 200 and body.get("status") == "cancelled" and calls[-1]["method"] == "DELETE"
          and calls[-1]["path"] == "/v1/subscriptions/7001", str((code, body)))
    check("cancelled stays open until the period ends", call("GET", "/api/courses/rust", token=tok)[1]["unlocked"] is True)
    code, _ = call("POST", "/api/admin/cancel", {"payment_id": s["id"]}, atok)
    check("can't cancel twice", code == 400)
    _, mine = call("GET", "/api/me/purchases", token=tok)
    _, pays = call("GET", "/api/admin/payments", token=atok)
    i2 = next(p for p in pays if p["provider_ref"] == "inv_9001")
    s2 = next(p for p in pays if p["provider_ref"] == "sub_7001")
    # Cancelled right after paying, period until 2031: nearly all unused, capped at what's left (500 - 200 refunded).
    check("unused-part refund suggested after cancelling", i2["prorata_cents"] == 300 and i2["prorata_total_days"] > 300
          and i2["cancelled_at"] and s2["cancelled_at"], str((i2.get("prorata_cents"), i2.get("prorata_total_days"), i2.get("cancelled_at"))))
    check("after cancelling, no cancel button", next(m for m in mine if m["id"] == s["id"])["can_cancel"] is False)
    expired = json.loads(json.dumps(sub))
    expired["meta"]["event_name"] = "subscription_expired"
    expired["data"]["attributes"].update({"status": "expired", "ends_at": "2020-01-01T00:00:00.000000Z"})
    check("expiry webhook", hook(expired) == 200)
    check("expired subscription closes the course", call("GET", "/api/courses/rust", token=tok)[1]["unlocked"] is False)

    # --- pause sales
    check("pause purchases", call("POST", "/api/admin/settings", {"sales_paused": True}, atok)[0] == 200)
    code, body = call("POST", "/api/payments/checkout", {"course_id": "sql", "plan": "monthly"}, tok)
    check("no checkout while paused", code == 400, str((code, body)))
    check("resume purchases", call("POST", "/api/admin/settings", {"sales_paused": False}, atok)[0] == 200)
    code, body = call("POST", "/api/payments/checkout", {"course_id": "sql", "plan": "monthly"}, tok)
    check("checkout after resuming", code == 200 and calls[-1]["body"]["data"]["relationships"]["variant"]["data"]["id"] == VAR_MONTHLY)

    # --- admin views
    _, users = call("GET", "/api/admin/users?q=buyer", token=atok)
    u = users[0] if users else {}
    check("admin sees phone and country", u.get("phone") == "+972501234567" and u.get("country") == "IL", str(u))
    check("admin sees payment method", "Visa •••• 4242" in u.get("payment_methods", []), str(u))
    _, ov = call("GET", "/api/admin/overview", token=atok)
    # The $25 order was a test-mode purchase (left out); the $5 invoice was live, $2 of it refunded.
    check("overview revenue: live money only, refunds separate",
          ov.get("revenue") == [["USD", 500]] and ov.get("refunded") == [["USD", 200]], str(ov))
    _, log = call("GET", "/api/admin/log", token=atok)
    actions = [e["action"] for e in log]
    check("every money action logged", actions.count("refund") == 3 and "cancel subscription" in actions
          and "pause purchases" in actions, str(actions))
    check("all fake API calls used the API key", all(c["auth"] == f"Bearer {API_KEY}" for c in calls))


if __name__ == "__main__":
    main()
