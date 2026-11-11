# egui_demo

This repository holds a hands-on egui 0.37 tutorial (a Todo List app) in `tutorial/`.

- Before writing or changing any tutorial here, read `.claude/tutorial-playbook.md`. It
  records the prompt, format, process, verification commands and the follow-on tutorials
  planned (database, HTTP, styling, Astryx to egui).
- `tutorial/README.md` is generated. Edit `.claude/tutorial-tools/README.tmpl.md` and the
  code under `tutorial/code/`, then rebuild:
  `python .claude/tutorial-tools/build_readme.py .claude/tutorial-tools/README.tmpl.md tutorial`
- egui 0.37 differs sharply from older examples. Check the installed crate source under
  `~/.cargo/registry/src/` before relying on remembered API.
