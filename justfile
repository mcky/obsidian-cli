build:
  cargo build

dev args:
  just find-proj-files | entr cargo run -- {{args}}

format:
  cargo fmt

format-watch:
  just find-proj-files | entr just format

test:
  cargo test

test-watch:
  just find-proj-files | entr just test

find-proj-files:
  find . -name "Cargo.toml" -o -name "*.rs"
