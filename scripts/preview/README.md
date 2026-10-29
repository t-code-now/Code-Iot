# Previews of a branch (before "תעלה")

The sandbox Claude works in cannot build Rust/WebAssembly and cannot reach the live
site, so previews use the real build made by CI:

1. Push the branch `preview/<name>` and run `build.yml` on it. On any branch other than
   `main` the workflow pushes the built web app to the branch `preview-dist`.
2. `git clone --depth 1 -b preview-dist https://github.com/t-code-now/Code-Iot.git dist`
3. `node scripts/preview/mock-api.js $PWD/dist 4100` – serves the build plus a small fake
   `/api` (one Python course with a lesson whose second step expects "14", a guest or a
   signed-in user "jean"). Extend the fake when a page needs more data.
4. `node scripts/preview/shots.js /login /lesson/py-1-2` (Playwright is preinstalled in
   the sandbox) – phone and desktop screenshots in Hebrew; `--signed-in` for a user.

The fake API is only for looking at pages. Payments are tested by `scripts/payments-test.py`.
