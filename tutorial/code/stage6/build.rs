fn main() {
    // `migrate!` embeds the SQL files at compile time, but Cargo only rebuilds when
    // Rust sources change. This makes a new or edited migration trigger a rebuild too.
    println!("cargo:rerun-if-changed=migrations");
}
