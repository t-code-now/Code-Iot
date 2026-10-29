# Working on Code-Iot – rules for Claude

Read this before doing anything in this repository. These rules come from the
owner and apply in every session, on every account.

## Talking with the owner
- Reply in **Hebrew**. Messages to outside organisations (e-mails etc.) are written in English.
- **Never** ask the owner to paste passwords, API keys, tokens or secrets in the chat.
  The owner puts secrets into Render and GitHub himself; tell him the variable *name* and where to put it.
- Never name or compare Code-Iot to any other learning app anywhere (code, README, texts,
  commit messages). Code-Iot is an independent product with its own name, design and content;
  never copy another app's text, images or branding.
- At the start of a new conversation, if the owner gives you the handoff document
  (`Code-Iot-handoff.txt`), read it fully, summarise the current state and the open tasks,
  and **change nothing** until he confirms.

## Preview first, deploy only on "תעלה"
Nothing reaches the live site or the app download link without the owner's explicit
approval. The approval word is **"תעלה"**.

1. Work on a branch `preview/<short-name>` made from `main`. Never commit straight to `main`.
2. Push the branch and run CI on it:
   `gh api -X POST repos/t-code-now/Code-Iot/actions/workflows/build.yml/dispatches -f ref=preview/<name>`
   (and `android.yml` too when Android code changed – on preview branches it only builds,
   it does not replace the download link).
3. Check the result: the `ci-log` branch (`build.log`, `summary.txt`) must show the build
   passed and **100 PASS** in the payments test. Android: `ci-log-android`.
4. Show the owner a preview: screenshots of the real build (the preview build of the web app
   is pushed to the `preview-dist` branch; serve it locally with a fake `/api` and capture
   phone 400px and desktop 1366px in Hebrew – see `scripts/preview/`), or a clear explanation when there is nothing
   visual. Say plainly what was and was not verified.
5. Only after "תעלה": `git merge --ff-only` the branch into `main` and push. Render deploys
   the site automatically; `android.yml` publishes the new APK. Wait for both runs and report.
   Tell the owner to wait ~15 minutes before checking the site (Render swaps the server).

## Do not change without asking first
- Prices, plans and what they open (`core/src/access.rs`, Lemon Squeezy products).
- Refund, cancellation, lock and dispute rules (`server/src/payments.rs`, `server/src/admin.rs`).
- Legal texts – Terms and Privacy (`web/src/pages/legal.rs`).
- Course content and access tiers (guest: 1 topic, signed-in: 2, paid/admin: all).
- Owner decisions already made: the "lock after refund" box is **not** pre-checked;
  monthly subscriptions show as a subscription row + a charge row in admin (keep as is);
  the website keeps Lemon Squeezy, Google Play Billing comes later for the app only.
- Python and Rust show 🐍/🦀 emoji until the PSF / Rust Foundation grant permission
  for their logos (requests sent 29.10.2026).
- Never delete users, payments or data, and never touch the live Lemon Squeezy store,
  without an explicit request.

## Things that must not break (learned the hard way)
- **Android signing key:** the APK is signed with the permanent keystore in GitHub Secrets
  (`ANDROID_KEYSTORE`, `ANDROID_KEYSTORE_PASSWORD`; SHA-1 `B1:EE:40:C2:9D:16:51:45:D5:6A:75:A6:DF:60:B4:6C:E8:FF:9B:E2`).
  Never generate a new key and never change the package id `com.tcodenow.codeiot`:
  updates would stop installing over the old app and in-app Google sign-in would break
  (the Android OAuth client in Google Cloud is tied to this SHA-1 and package).
- **Two languages everywhere:** every UI string needs English and Hebrew (`web/src/i18n.rs`);
  every lesson text needs its Hebrew line (`cargo test -p codesprout-core` fails otherwise).
  Hebrew is right-to-left; code always stays left-to-right.
- **Check phone and desktop:** layouts differ (phone first, tablet 640–1023px, desktop ≥1024px).
  Check both, in Hebrew, and the Android app when the change can affect it.
- **Consumer-protection rules already implemented – keep them:** prices shown include tax,
  14-day full refund, self-service cancellation page `/cancel` (cancel a subscription in a
  few taps, request a refund), clear subscription terms, seller-details slot in the Terms
  (`BUSINESS` in `legal.rs`, filled once the owner registers a business).
- **Payments test:** when payment logic changes, update `scripts/payments-test.py` (and its
  fake Lemon Squeezy) so it still covers every flow; all 100+ checks must pass.
- **Test mode vs live:** the store is in Lemon Squeezy test mode and purchases are paused in
  admin until the owner goes live. Admin overview totals count real payments only (on purpose).
- **Design:** follow the design system in `docs/design-system/` (README in Hebrew + `tokens.json`)
  and the existing styles in `web/style/main.css`: dark navy panels on the painted sky,
  one gold action colour, solid 3-D button edges, no small text straight on the clouds.
  Don't add new fonts, colours or component styles without asking.
- **Data is live:** users, progress and payments are in Turso. Never reset, drop or rewrite
  tables; schema changes must keep existing rows.
- **The address `code-iot.onrender.com` is wired into several places:** Google OAuth redirect
  URIs (Google Cloud), the Lemon Squeezy webhook URL, `PUBLIC_URL`, the Android build
  (`API_URL`, default in `android.yml`). Moving to another domain means updating all of them
  (the owner does the dashboards).
- **Lemon Squeezy webhook:** URL `https://code-iot.onrender.com/api/payments/webhook`, signed
  with `LEMONSQUEEZY_WEBHOOK_SECRET`. The code uses: `order_created`, `order_refunded`,
  `subscription_*` (created/updated/cancelled/resumed/expired/paused/unpaused),
  `subscription_payment_*` (success/failed/refunded), `dispute_created`, `dispute_resolved`.
  When going live, the live webhook must send all of them. Monthly plans come from
  subscription events, yearly plans from order events (matched by variant id).
- **Google OAuth:** website redirect URI `https://code-iot.onrender.com/api/auth/google/callback`
  (Web client); the app uses the Android client (package + SHA-1 above) and sends the ID
  token to `/api/auth/google/id-token`.
- **Playground** runs Python/C++/Rust/Arduino through the public Compiler Explorer API
  (godbolt.org); if it changes or limits requests, the playground breaks.
- **Google sign-in consent screen** (Google Cloud → Google Auth Platform → Audience) is
  "In production", External (since 27.10.2026). Never move it back to "Testing" – then only
  listed test users could sign in.
- **Keepalive:** `.github/workflows/keepalive.yml` pings `/api/health` every 4 minutes so the free Render server
  does not sleep. Don't remove it without a replacement.
- **The repository is public for now** (the APK download link and the keepalive workflow rely
  on that). Do not make it private before the planned steps are done: APK served from our
  server, keepalive moved to an external pinger, Render connected through GitHub.
- **This environment's limits:** Rust/WASM cannot be built locally and the live site is not
  reachable from the sandbox; GitHub GraphQL, the Actions secrets API and artifact downloads
  are blocked. That is why previews go through CI and the `preview-dist` branch.

## Project map
- `core/` content (7 courses, 719 lessons, Hebrew in `core/src/content/he/`), grading, XP,
  access rules, API types. No dependencies.
- `server/` axum API, SQLite/Turso, payments (Lemon Squeezy webhooks), admin. Serves `web/dist`.
- `web/` Leptos 0.6 → WebAssembly (Trunk). UI strings: `web/src/i18n.rs` (key, English, Hebrew).
  Styles: `web/style/main.css` (phone first; tablet 640–1023px; desktop ≥1024px).
- `app/` Tauri v2 Android app (outside the workspace) + `app/google-signin` plugin
  (in-app Google sign-in, keeps the app clear of the status bar).
- `scripts/payments-test.py` end-to-end payments test against a fake Lemon Squeezy.
- `scripts/preview/` tools for preview screenshots (fake API + Playwright).
- `docs/design-system/` the design system (Hebrew README + tokens).
- Live site: https://code-iot.onrender.com · Admin: /admin (Google account tamirmaidani@gmail.com).
- Full history, decisions and open tasks: the owner's handoff document.
