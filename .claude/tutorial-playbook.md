# Tutorial playbook

How the egui Todo List tutorial in `tutorial/` was produced (November 2026), written so the
same process and format can be repeated for follow-on tutorials.

## Current state (saved 2026-11-14)

- **Done and verified:** Hello World and stages 1 to 6. The tutorial pages are generated and
  match the code. Workspace is clean under fmt, clippy and tests (3, 7, 19, 32, 44).
- **Split into chapters (2026-11-14, user request):** `tutorial/README.md` is now a contents
  page; each chapter is its own page under `tutorial/chapters/`, with generated
  previous/contents/next links, so readers on GitHub navigate instead of scrolling.
- **What the user is doing now:** working through the tutorial by hand. Expect feedback,
  corrections or questions about specific stages when they return. Fix problems in the code
  under `tutorial/code/` and in the templates under `.claude/tutorial-tools/`, then rebuild;
  never edit the generated pages under `tutorial/` directly.
- **Concept pages (2026-11-14):** eight in-depth pages for the types named in Hello World
  (`central-panel`, `ui`, `native-options`, `viewport-builder`, `run-native`, `app-trait`,
  `layout`, `response`) under `tutorial/concepts/`, each linked from the Hello World table and
  linking back, with runnable examples in the `concepts` crate. Verified; see "Concept pages
  and the concepts crate". A second batch is still to do (listed there).
- **Next planned work, not started:** stage 7 (much better styling), then the Astryx to egui
  conversion tutorial. Astryx has not been researched. Both need a verification plan
  confirmed by the user before building.
- **Git:** the work is committed on `main` and pushed to `origin/main`. The GitHub
  repository was renamed to `eframe-egui-tutorial`.
- **Machine state:** the app-data folder `%APPDATA%\egui-todo-tutorial` was removed after the
  last checks, so the user's first run of stage 4, 5 or 6 starts from the sample todos.
  Running stage 4 or 5 and then stage 6 is fine: they share the folder, and stage 6 ignores
  the todos in the older save file.

## The original request

The tutorial came from this prompt template. Reuse it by swapping the bracketed values.

```text
Act as a senior software engineer and technical writer. Write a comprehensive, hands-on coding
tutorial for the **[egui]** library (version **[0.37]**) in **[Rust]**. Library documentation
found here: **[https://crates.io/crates/egui]**

### Target Audience & Context
* **Skill Level:** [Experienced developer new to this library]
* **Prerequisites:** [Working knowledge of Rust]
* **Goal:** By the end of this tutorial, the reader will build **[A windows application to a
  Todo List, that uses all of egui features, and re-usable components built with best practices
  for scalability and re-usablity]**.

### Tutorial Structure
1. **Introduction:** Briefly explain what the library does, why developers use it, and what we
   are building.
2. **Setup & Installation:** Exact terminal commands for installation and any required
   environment variables or configuration.
3. **Core Concepts (The "Hello World"):** The simplest possible working example. Explain each
   line briefly.
4. **Step-by-Step Project Build:** Build the project in 3 to 4 logical stages. Include
   complete, runnable code blocks for each stage.
5. **Common Pitfalls & Best Practices:** 3 common beginner mistakes and how to avoid them.
6. **Next Steps:** 2 small feature extensions the reader can build on their own.

### Formatting & Coding Rules
* Clean, idiomatic code with descriptive variable names and inline comments.
* No placeholder comments like `// add logic here`; write out the full working code.
* Keep explanations concise, focusing on *why* a method or pattern is used.
```

It was followed by: "Write this into its own directory in this current directory, called
tutorial".

## What exists

| Path | What it is |
|---|---|
| `tutorial/README.md` | The contents page: intro, "Tested against" box, every chapter with its sections, the concept pages. **Generated; do not edit by hand.** |
| `tutorial/chapters/NN-name.md` | The tutorial, one page per chapter (`01-introduction` to `11-next-steps`; stages 1 to 6 are `04` to `09`). **Generated.** |
| `tutorial/concepts/` | In-depth pages for the types named in Hello World, plus their index. **Generated.** |
| `tutorial/code/` | Cargo workspace: `hello`, `stage1` to `stage6`, `concepts` (gallery, examples, tests) |
| `tutorial/images/` | Real window screenshots, at least one per crate |
| `.claude/tutorial-tools/README.tmpl.md` | Source of the contents page (hand-written contents list, `@@concept-links`) |
| `.claude/tutorial-tools/chapters/*.tmpl.md` | Source of each chapter (prose plus `@@file` and `@@excerpt` includes) |
| `.claude/tutorial-tools/concepts/*.tmpl.md` | Source of each concept page |
| `.claude/tutorial-tools/build_readme.py` | Builds every page, inserts chapter navigation, and verifies pages against the code |
| `.claude/tutorial-tools/test_build_readme.py` | Unit tests for the build script (`python -m unittest`) |
| `.claude/tutorial-tools/shot.ps1` | Launches an exe, screenshots its window, reports if it stayed alive |
| `.claude/tutorial-tools/drive.ps1` | Same, but also clicks at window-relative pixels between screenshots |

## The format

- **One runnable crate per stage** in a single workspace, so every stage's code compiles and
  can be run with `cargo run -p stageN`. Later stages are full copies, not diffs.
- **The README never contains hand-typed code.** Excerpts come in through
  `@@excerpt <path> ||| <start text> ||| <end text> ||| <extra lines>`, which copies the
  lines from the source at build time, so they cannot go stale. (The stage 1 to 4 excerpts
  were pasted by hand before this existed; new ones should use the directive.)
  Whole files come in through `@@file <path>`
  lines in the template. Excerpts are pasted from the source, and the build script fails if
  any ` ```rust ` block is not found verbatim in a `.rs` file. Use ` ```rust,ignore ` for
  illustrative snippets that are not meant to match.
- **One page per chapter.** `chapters/NN-name.tmpl.md` is read in file-name order. Each has
  exactly one `# ` title (no "1." numbering; the contents page numbers the chapters) and its
  sections start at `##`. Templates hold no navigation: the build inserts
  `[Previous: <title>](..) · [Contents](../README.md) · [Next: <title>](..)` under the title
  and at the end. Pages sit one folder down, so images are `../images/..`, code links
  `../code/..`, concept links `../concepts/..`, and a link to a heading in another chapter is
  `NN-name.md#anchor`. `@@file`/`@@excerpt` paths stay relative to `tutorial/`. To add a
  chapter (for example stage 7), add a template with the right number and add it, with its
  `##` sections, to the contents list in `README.tmpl.md`; the build fails if a chapter is
  missing from the contents or listed out of order.
- **Per stage:** what the stage adds and why, a screenshot, new or heavily changed files in
  full, small changes as exact excerpts, then a short "what to notice" on the design choice.
- **Chapter `10-pitfalls`** has the three pitfalls, plus a migration table for readers coming from older
  versions and a list of the best practices the code follows.
- **Scope is stated openly.** "All features" was scoped to a feature table in the
  introduction with an explicit "what is left out" list.
- **Web way, then egui way** (user feedback, 2026-11-14; the user values the web/CSS
  comparisons highly): wherever a feature has a real HTML, CSS or JavaScript equivalent, show
  the web block (` ```css `/` ```html `/` ```js `, hand-typed, kept minimal and correct) followed
  by `**The egui way:**` with the real excerpt, each with a one-line lead saying what differs.
  Label the web block by its language: `**The CSS way:**` when it is CSS (optionally with
  HTML), `**The JS way:**` when it has JavaScript (optionally with HTML), `**The HTML way:**`
  when it is HTML only.
  This applies to standard usage and every micro-example with an equivalent, as well as the
  "If you come from the web" box. Skip it where no honest equivalent exists; never invent one.
  **Web platform only** (user, 2026-11-14): no references to or comparisons with desktop
  web-wrapper frameworks; compare with the browser and web platform only.
- **Every diagram comes twice** (user feedback, 2026-11-14): a ` ```text ` block, readable in
  any viewer, followed straight after by a ` ```mermaid ` block for the same picture (`block-beta`
  for layouts, `flowchart` for data flow, `sequenceDiagram` for threads and messages). Avoid
  `;` and raw `<`/`>` in Mermaid labels (use `#lt;`/`#gt;`). File trees and command output are
  listings, not diagrams: text only.
- Dependencies are declared once in `[workspace.dependencies]`; a feature only one stage needs
  (such as `persistence`) is enabled in that stage's `Cargo.toml`.

## Concept pages and the concepts crate

Added 2026-11-14. The Hello World chapter names eight types; each now has its own page.

**Page format** (copy `.claude/tutorial-tools/concepts/central-panel.tmpl.md`, the approved
pattern; 150 to 500 lines):

- Heading order of the pattern page, a back link to `../chapters/03-hello-world.md` under the
  title, and an "If you come from the web" box.
- A "Standard usage" part, then micro-examples. Each micro-example has `**Use this when**`,
  the pairs, then `Run:` (the `cargo run` command) and a short note.
- **Web way, then egui way** pairs wherever an honest HTML, CSS or JavaScript equivalent
  exists: hand-typed ` ```css `/` ```html `/` ```js `, then the real `@@excerpt`. Label the web
  block by its language: `**The CSS way:**` (CSS, optionally with HTML), `**The JS way:**`
  (JavaScript, optionally with HTML) or `**The HTML way:**` (HTML only). Skip the pair where
  there is no equivalent.
- Every diagram twice: a ` ```text ` block, then a ` ```mermaid ` block of the same picture.
- No ` ```rust ` typed by hand: `@@excerpt` or `@@file` only. ` ```rust,ignore ` only for
  fragments that cannot compile alone.
- No references to or comparisons with desktop web-wrapper frameworks; compare with the
  browser and web platform only. No emoji or symbol glyphs in example UI.
- **Images must not show machine or personal details** (user name, `C:\Users\<name>` paths,
  GPU name, hostname). The renderer example prints only the graphics API and device type.
  Re-read every screenshot before keeping it.
- Every API name, default and behaviour is checked against the installed `egui-0.37.0`,
  `eframe-0.37.0` and `egui-winit-0.37.0` source; leave out what the source does not confirm.

**The `concepts` crate** (`tutorial/code/concepts`, a workspace member):

- `src/` holds a **gallery** (`gallery.rs`): a registry of small examples, one module per page
  (`central_panel.rs`, `ui_region.rs`, `layout.rs`, `response.rs`, `app_trait.rs`). Run it with
  `cargo run -p concepts`, or `cargo run -p concepts -- "Panel order"` to open at one example.
  Pages excerpt from these modules.
- `examples/` holds the examples that need their own window or `NativeOptions`
  (`cargo run -p concepts --example <name>`): the `native_options_*`, `viewport_builder_*`,
  `run_native_*`, `app_trait_*` ones and `central_panel_root`. Each sets its own window title,
  used by `shot.ps1 -Title`; `central_panel_root` and `run_native_ui_only` use the app name
  passed to `run_native` / `run_ui_native` instead.
- `tests/` has headless `egui_kittest` tests per page. Examples with their own unit tests are
  listed in `Cargo.toml` as `[[example]] test = true`.
- **`persistence` feature:** turns on `eframe/persistence`. Only `native_options_persist`,
  `run_native_restore` and `app_trait_save_and_exit` need it (`required-features`). Their
  tests run only with `--all-features`. An exe built with `--all-features` writes a folder
  under `%APPDATA%` even if it is a different example, so list `egui*` there before launch
  checks and delete whatever is new afterwards.

**Build script** (`build_readme.py`) now builds the contents page, `chapters/*.md` and
`concepts/*.md`. It inserts chapter navigation, checks every ` ```rust ` block against the
sources, checks images, and reports `broken links: []` and `chapter problems: []`.
`test_build_readme.py` (17 tests) covers it. Concept pages sit in `tutorial/concepts/`, so
links to chapters are `../chapters/NN-name.md` and images are `../images/concepts/x.png`.

**Chapter split:** see "One page per chapter" under "The format". The Hello World table lives in
`.claude/tutorial-tools/chapters/03-hello-world.tmpl.md`; all eight concept rows link to
`../concepts/<page>.md`.

**Verified 2026-11-14:** fmt, check, clippy (`-D warnings`) and `cargo test --workspace` all
pass; stage tests unchanged (3, 7, 19, 32, 44); the concepts crate has 91 tests (99 with
`--all-features`); the build is clean on every page. The gallery and all 25 binaries in
`examples/` launch and stay alive under `shot.ps1`.

**Windows link flake:** `cargo test --workspace` or `cargo build --examples` sometimes fails
with `LNK1104: cannot open file ...\examples\<name>.exe` when many example exes link at once
(no process held them; likely a scanner). Re-running succeeds. Not a code problem.

**Still to do:** a second batch of concept pages for the remaining types and calls used in
the stages (for example `Panel`, `ScrollArea`, `Table`, `Context`/`Id`, `Frame`/`Style`,
`MenuBar`, `Window`/`Modal`); the user has not chosen the list. The never-exercised list in
"Verification commands" still applies, and no concept example was driven with clicks
(`drive.ps1` was not used for them).

## The process

1. **Verification plan first**, confirmed by the user before building (their standing rule).
2. **Check the API against the installed source**, not memory or older examples. The crates
   are under `~/.cargo/registry/src/*/egui-0.37.0` and siblings. This mattered a lot: see
   "egui 0.37 facts" below.
3. Build stage by stage. After each: `cargo check`, `cargo clippy`, run it, screenshot it.
4. Write prose in the templates, generate the pages, fix whatever the script reports.
5. Run the full verification below and report the results, including what was not exercised.

## Verification commands

Run from `tutorial/code` unless noted. All must pass before the tutorial is called done.

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
# the concepts crate has a cargo feature, so also with it on:
cargo check -p concepts --all-targets --all-features
cargo clippy -p concepts --all-targets --all-features -- -D warnings
cargo test -p concepts --all-features
```

From the repository root, test the build script, then regenerate and check every page (the
contents page, each chapter and each concept page):

```sh
python -m unittest discover -s .claude/tutorial-tools -p "test_*.py"
python .claude/tutorial-tools/build_readme.py .claude/tutorial-tools/README.tmpl.md tutorial
```

The build must exit 0, with 0 blocks not found and no missing images on every page,
`broken links: []` and `chapter problems: []`.

Launch check and screenshot for each binary (PowerShell, from the repository root):

```powershell
.claude/tutorial-tools/shot.ps1 -Exe tutorial/code/target/debug/stage4.exe -Out tutorial/images/stage4.png -Title "Todo List"
```

`-Title` must match the window title (`"Hello egui"` for `hello`). Add `-Graceful` to close
with WM_CLOSE so the app saves, which is how the persistence round trip was checked.

Results on 2026-11-11: stage 2 has 3 tests, stage 3 has 7, stage 4 has 19, stage 5 has 32,
stage 6 has 44, all passing. The `concepts` crate has 91 (99 with `--all-features`). Stages 4 to 6 include headless UI tests through `egui_kittest`.

**Clean up after launch checks.** Running the apps creates folders under `%APPDATA%`
(`egui-todo-tutorial`, and others named after the app name or app id, not the window title). Stage 4's save file is
`%APPDATA%\egui-todo-tutorial\data\app.ron`. Remove them so the reader's first run starts
from the sample data.

Never exercised in a window or a test: double-click to edit, Ctrl+N and Ctrl+F, collapsing the
sidebar, resizing table columns, the look of the accent colour and row spacing settings, and
the Stop button during a real (not mocked) download.

**Driving a real window.** `drive.ps1` moves the real mouse and takes focus, so it disturbs
the user if they are at the machine, and their keystrokes can land in the app. Example
(coordinates measured on a 776x559 screenshot of the stage 5 window):

```powershell
.claude/tutorial-tools/drive.ps1 -Exe tutorial/code/target/debug/stage5.exe -Steps "click:27,43;wait:500;click:78,95;wait:700;click:570,346;wait:9000;shot:tutorial/images/stage5.png"
```

**Antivirus prompt (reported by the user, 2026-11-11).** The first network connection from a
newly built executable makes the user's antivirus ask them to approve it. The request waits
until they answer and the prompt can take focus from scripted clicks. Do not try to bypass
it: finish code changes first, build once, tell the user a prompt is coming, then run the
network check. Tests are unaffected because they use a local mock server.

## egui 0.37 facts that differ from older material

- `eframe::App` requires `ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`.
  `update(ctx, ..)` is gone. The `run_native` creator closure returns a `Result`.
- `TopBottomPanel` and `SidePanel` are replaced by `egui::Panel::top/bottom/left/right`, shown
  into a `ui`. `menu::bar` is replaced by `egui::MenuBar::new().ui(..)`.
- `Id::new` is deprecated; use `Id::unique` for global IDs and salts for local ones.
- **Emoji and symbol glyphs are not bundled.** They render only where a system font has them
  and panic in headless `egui_kittest` ("No glyph for ..."). egui's own
  `global_theme_preference_buttons` hits this. The tutorial uses text buttons, a hand-painted
  drag handle and a custom `theme_picker`. The `monochrome_emoji_fonts` feature brings the
  glyphs back if wanted.
- `egui_kittest` fails a test when an interactive widget has no accessible name, so
  icon-style buttons need `accessible_name(..)`. `type_text` types into whatever has focus;
  focus the field and run a frame first (see the `type_into` helper in stage 4).
- Toolchain used: rustc 1.99.1, edition 2024, resolver 3, Windows 11.

## Architecture the tutorial teaches

Later tutorials should build on this rather than restart.

- The model (`model.rs`) has no egui dependency and carries the unit tests.
- Components never mutate the model. They return a `TodoAction`; the app collects actions
  while drawing and applies them afterwards in `TodoApp::apply`.
- Three component shapes: a leaf `impl Widget`, a composite `show(self, ui) ->
  Option<TodoAction>`, and a stateful struct with `show(&mut self, ui)`.
- `PersistedState` (saved, `#[serde(default)]`) is kept apart from transient UI state.
- `TodoApp::show(ui)` is separate from `App::ui` so tests can drive the whole app.

## Planned follow-on tutorials

Requested by the user. Stages 5 and 6 are done; 3 and 4 below are not started.

1. **Stage 5: HTTP requests. DONE 2026-11-11.** User's decisions: `reqwest`, placed before
   the database stage, importing from `https://jsonplaceholder.typicode.com/todos`.
   Built: `import.rs` (request, wire type, `ImportError`), `worker.rs` (one thread with a
   current-thread tokio runtime; `Command` in, `Event` out, a `wake` closure for
   `request_repaint`), `components/import_dialog.rs`, a request ID to drop stale answers.
   Tests use `wiremock`; UI tests step frames with a `step_until` helper because a visible
   spinner never lets `harness.run()` settle.
2. **Stage 6: database. DONE 2026-11-11.** User's decisions: `sqlx` with SQLite, using
   runtime-checked `query_as` with `FromRow`, not the `query!` macros.
   Built: `storage.rs` (`Repository` over a one-connection `SqlitePool`, `TodoRecord` row
   type), `migrations/0001_create_todos.sql`, `build.rs` (rebuild when migrations change),
   database commands on the stage 5 worker (awaited in order; the import stays spawned).
   Todos live in `todos.db` beside the eframe save file; preferences stay in the eframe
   save. The in-memory list is changed first, then the write is queued. A new database is
   seeded with the sample todos; stage 4 saves are not imported.
   sqlx 0.9 notes: `migrate!` needs the `macros` feature; an in-memory pool needs
   `max_connections(1)` with idle timeout and max lifetime disabled.
   Versions used: reqwest 0.13.5, sqlx 0.9.0, tokio 1.54.1, wiremock 0.6.5.
3. **Much better styling (next, would be stage 7).** Stage 4 only has an accent colour,
   row spacing and theme choice.
4. **Converting Astryx to egui.** Docs: https://astryx.atmeta.com/docs/getting-started.
   Astryx has not been researched yet; read those docs before planning.

For each: same prompt template, same six sections, same one-crate-per-stage workspace, same
generated pages and verification commands.

**Decided by the user (2026-11-11): follow-on tutorials continue as extra stages.** Add
`stage5`, `stage6`, ... as new crates in the `tutorial/code` workspace, each starting as a
full copy of the previous stage, and add a chapter template for each (renumbering the
`10-pitfalls` and `11-next-steps` templates so they stay last) rather than starting a
separate tutorial directory.
