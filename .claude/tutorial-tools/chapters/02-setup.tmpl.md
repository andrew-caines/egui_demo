# Setup & installation

You need a Rust toolchain (1.95+) with the MSVC build tools, which `rustup` installs by
default on Windows. No environment variables or extra configuration are required.

```powershell
rustc --version          # must print 1.95.0 or newer; run `rustup update` if not

cargo new todo_app
cd todo_app
cargo add eframe@0.37
```

Later stages add more dependencies. You can add them now or when you reach the stage:

```powershell
# Stage 3: tables
cargo add egui_extras@0.37

# Stage 4: saving state, and headless UI tests
cargo add eframe@0.37 --features persistence
cargo add serde --features derive
cargo add --dev egui_kittest@0.37

# Stage 5: HTTP requests, and a local mock server for the tests
cargo add reqwest@0.13 --features json
cargo add tokio@1 --features rt,sync
cargo add --dev wiremock@0.6
cargo add --dev tokio@1 --features macros

# Stage 6: SQLite
cargo add sqlx@0.9 --no-default-features --features runtime-tokio,sqlite,migrate,macros
cargo add --dev tempfile@3
```

Stage 5 makes real network requests. The first time a newly built executable connects,
Windows security software may ask you to allow it, and the request waits until you answer.

The first build compiles the renderer and takes a few minutes. Later builds take seconds.

## Running the companion code

Every stage is also a ready-to-run crate in this directory:

```powershell
cd tutorial/code
cargo run -p hello
cargo run -p stage1      # ... stage2 to stage6
cargo test --workspace
```
