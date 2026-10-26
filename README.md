# Code-Iot 🌱

A learn-to-code app in the style of bite-sized coding apps, written entirely in Rust:

- **Backend:** axum + SQLite (accounts, progress, XP, streaks, badges, leaderboards)
- **Frontend:** Leptos compiled to WebAssembly, mobile-first dark UI
- **Core:** dependency-free crate with all course content and game logic, shared by both

**English and Hebrew (עברית).** The whole interface and all 719 lessons are available in both languages, with a full right-to-left layout for Hebrew. Switch from the welcome screen or Profile. Code always stays in English and left-to-right, as in a real editor.

**Everything is open.** No locked lessons, no hearts/energy, no paywall. Every course and every lesson can be opened from the start. An account is only needed to save progress; you can also try it as a guest.

## Features

- 7 complete courses — 719 lessons (at least 100 per course) and 3,159 steps (teaching cards and exercises). Every topic ends with a project lesson that uses that topic's own material:

  | Course     | Sections | Lessons | Covers |
  |------------|---------:|--------:|--------|
  | Python     | 18 | 104 | basics, strings, logic, loops, lists/tuples, dicts/sets, functions & recursion, errors/files/modules, **OOP** (classes, inheritance, polymorphism, dunder methods, properties, abstract classes), generators, decorators, dataclasses, type hints, text processing & regex, the standard library, functional Python, context managers, testing & debugging, algorithms & data structures, project |
  | JavaScript | 18 | 102 | basics, logic, loops, functions & closures, arrays, objects & JSON, **OOP** (classes, getters/setters, inheritance, static/private, prototypes, polymorphism), errors, modules, async/promises/fetch, DOM, Map/Set, localStorage, strings & regex, advanced functions, iterators & generators, advanced async, testing & debugging, data structures & algorithms, project |
  | HTML & CSS | 19 | 101 | HTML structure, semantic HTML, forms, selectors, cascade, box model, flexbox, grid, responsive design, transitions, animations, variables, accessibility, advanced selectors, typography & web fonts, images & media, advanced layout, modern CSS, forms in depth, CSS architecture, publishing, project |
  | SQL        | 19 | 101 | SELECT, filtering, sorting, CASE, aggregates, GROUP BY/HAVING, joins, subqueries, CTEs, INSERT/UPDATE/DELETE, table design, keys, indexes, transactions, views, window functions, text, dates & times, NULL logic, advanced joins, recursive queries, upserts & JSON, query performance, security, project |
  | Rust       | 19 | 101 | basics, control flow, ownership & borrowing, structs, enums & match, collections, error handling, **traits & generics** (Rust's OOP), closures, iterators, lifetimes, smart pointers, strings, collections/iterators/traits in depth, modules & testing, concurrency, files & CLI tools, async, project |
  | C++        | 17 | 102 | basics, decisions, loops, functions, arrays & strings, vectors, references & pointers, structs & enums, **OOP** (classes, inheritance, polymorphism), memory & smart pointers, operator overloading, copy & move, templates & modern C++, STL containers, lambdas & algorithms, exceptions, files & streams, project |
  | Arduino    | 18 | 108 | **the full Arduino language from scratch** (syntax, every data type, literals, casting, all operators incl. bitwise, control flow, functions/scope/static/references/overloading/recursion, arrays, C strings & String, pointers, structs, enums, built-in library, preprocessor, multi-file sketches, PROGMEM, **OOP**: classes, constructors, encapsulation, inheritance, virtual/polymorphism, static, templates), then electronics, digital & analog I/O, PWM, sound, servos, motors, Serial/I²C/SPI, sensors, interrupts, state machines, EEPROM, project |

  Every section ends with a project, and each course ends with a final project you build step by step.
- Winding lesson path per course with sections, checkmarks and a "START" marker
- Lesson player: teaching cards, multiple choice, tap-to-fill code blanks, line reordering, type-in answers
- Check button with green/red feedback sheet, correct answer and explanation
- Lesson-complete screen: XP, accuracy, streak, newly unlocked badges
- XP and levels, daily streaks (timezone-aware), 8 badges
- Weekly league + all-time leaderboard
- HTML/CSS/JS playground running in a sandboxed iframe
- Sign up / log in (Argon2 password hashing, bearer-token sessions)
- English / Hebrew with right-to-left layout and a language switcher

## Project layout

```
codesprout/
├── core/     # content, grading, XP/levels/streaks/badges, API types (no deps)
├── server/   # axum API + SQLite, serves the built frontend
└── web/      # Leptos WASM app (Trunk), style/main.css
```

## Requirements

- Rust (stable) — https://rustup.rs
- WebAssembly target: `rustup target add wasm32-unknown-unknown`
- Trunk: `cargo install trunk`

## Run it

Development (two terminals):

```bash
# 1. API on :3000
cargo run -p codesprout-server

# 2. Frontend on :8080 with hot reload (proxies /api to :3000)
cd web && trunk serve
```

Open http://localhost:8080. To try it on your phone, use your computer's LAN IP, e.g. `http://192.168.1.20:8080`.

Production (one server serves everything):

```bash
cd web && trunk build --release && cd ..
cargo run -p codesprout-server --release
# → http://localhost:3000
```

Server settings (environment variables):

| Variable        | Default          |
|-----------------|------------------|
| `ADDR`          | `0.0.0.0:3000`   |
| `DATABASE_PATH` | `codesprout.db`  |
| `WEB_DIST`      | `web/dist`       |

## Tests

```bash
cargo test -p codesprout-core     # content validity, grading, XP, streaks, badges, shuffle
cargo test -p codesprout-server   # database layer
```

## API

| Method | Path                          | Auth     | Purpose                           |
|--------|-------------------------------|----------|-----------------------------------|
| POST   | `/api/auth/signup`            |          | Create account → token + profile  |
| POST   | `/api/auth/login`             |          | Log in → token + profile          |
| POST   | `/api/auth/logout`            | optional | End session                       |
| GET    | `/api/me`                     | required | Profile (XP, level, streak…)      |
| GET    | `/api/courses`                | optional | Courses with your progress        |
| GET    | `/api/courses/:id`            | optional | Sections and lessons              |
| GET    | `/api/lessons/:id`            |          | Lesson steps (no answers)         |
| POST   | `/api/lessons/:id/check`      |          | Check one answer                  |
| POST   | `/api/lessons/:id/complete`   | required | Save completion, award XP         |
| GET    | `/api/leaderboard?scope=weekly\|all_time` | optional | Rankings              |

Auth uses `Authorization: Bearer <token>`. Content endpoints (`/api/courses`, `/api/courses/:id`, `/api/lessons/:id`, `/api/lessons/:id/check`) accept `?lang=he` for Hebrew; English is the default.

## Adding content

Each course lives in its own file under `core/src/content/` (python.rs, javascript.rs, web.rs, sql.rs, rust.rs, cpp.rs, arduino.rs + arduino_lang.rs). Add a `lesson(...)` with steps built from
`learn`, `mc`, `fill` (use `___` for each blank), `order` (lines in the correct
order — they're shuffled when served) and `typein`. Run `cargo test -p codesprout-core`;
the content tests catch broken exercises (missing tiles, bad indexes, duplicate ids).

## Note

Code-Iot is an independent project. It is not affiliated with or endorsed by Mimo,
and uses its own name, design and course content.

## License

MIT

## Translations

- **Course content:** `core/src/content/he/<course>.txt`, one `key = value` line per string
  (keys like `py-1-1.0.body`; see `core/src/i18n.rs` for the format). Only prose is
  translated; code, tiles and accepted answers are shared by all languages.
  `cargo test -p codesprout-core` fails if a key is wrong or any prose string is missing.
- **Interface:** `web/src/i18n.rs`, a table of `(key, English, Hebrew)`.
- **Adding a language:** add a `Lang` variant, a `content/<lang>/` folder, and a column in the UI table.
