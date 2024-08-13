_default:
  @just --list

build:
  cargo build

run args:
  cargo run -- {{args}}

dev args:
  just find-proj-files | entr just run '{{args}}'

format:
  cargo fmt

format-watch:
  just find-proj-files | entr just format

test args="":
  cargo test {{args}}

test-watch args="":
  just find-proj-files | entr just test {{args}}

find-proj-files:
  find . -name "Cargo.toml" -o -name "*.rs"