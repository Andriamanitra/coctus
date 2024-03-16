# Build using cargo
build:
    cargo build

# Format code using rustfmt
format:
    cargo +nightly fmt

# Install `clash` binary
install:
    cargo install --path .

# Run tests
test:
    cargo test

# Run painting tests
test-painting:
    cargo test --quiet painting -- --nocapture --test-threads=1

# Print unformatted statement (requires clash & jq)
raw-statement HANDLE:
    clash json {{HANDLE}} | jq .lastVersion.data.statement

# Check if clashes look good
check-all: check-outdated check-mono check-nested check-nested-self check-not-matching

# Check outdated formatting
check-outdated:
    cargo run --quiet -- show 25623694f80d8f747b3fa474a33a9920335ce | less -R
    cargo run --quiet -- show 7018d709bf39dcccec4ed9f97fb18105f64c  | less -R

# Check Monospace
check-mono:
    cargo run --quiet -- show 1783dda5b69105636695dc5bf51de1baf5d0  | less -R
    cargo run --quiet -- show 1222536cec20519e1a630ecc8ada367dd708b | less -R
    cargo run --quiet -- show 6357b99de3f556ffd3edff4a4d5995c924bb  | less -R
    cargo run --quiet -- show 4730251b4f27c2549cffc6fa48c40b7b85c8  | less -R

# Check tags nesting
check-nested:
    cargo run --quiet -- show 750741cba87bb6a6ac8daf5adbe2aa083e24  | less -R
    cargo run --quiet -- show 83316b323da5dba40730dbca5c72b46ccfc9  | less -R

# Check self nesting
check-nested-self:
    cargo run --quiet -- show 70888dd5bb12f2becdad5e6db3de8b40a77f  | less -R

# Check not matching tags
check-not-matching:
    cargo run --quiet -- show 7040402a6fe461068f5cf5296607c184d043a | less -R

###################
# HERE BE DRAGONS #
###################

# Change this to your favorite text-editor
editor := "code"

launch-rb:
    cargo run --quiet -- generate-stub "ruby" > tmp.rb
    {{editor}} tmp.rb
    cargo run --quiet -- show
    ls *.rb | entr -p cargo run --quiet -- run --command "ruby tmp.rb"

launch-new-rb:
    cargo run --quiet -- next
    cargo run --quiet -- generate-stub "ruby" > tmp.rb
    {{editor}} tmp.rb
    cargo run --quiet -- show
    ls *.rb | entr -p cargo run --quiet -- run --command "ruby tmp.rb"

launch-py:
    {{editor}} tmp.py
    cargo run --quiet -- show
    ls *.py | entr -p cargo run --quiet -- run --command "python3 tmp.py"

launch-c:
    {{editor}} tmp.c
    cargo run --quiet -- show
    ls *.c | entr -p cargo run --quiet -- run \
    --build-command "gcc -o tmp tmp.c" --command "./tmp"

# Requires Cargo.toml to look be like this:
# [package]
# name = "clash"
# version = "0.1.0"
# edition = "2021"
# default-run = "clash"

# [[bin]]
# name = "tmp"
# path = "tmp.rs"
launch-rs:
    {{editor}} tmp.rs
    cargo run --quiet -- show
    ls *.rs | entr -p cargo run --quiet -- run \
    --command "cargo run --bin tmp"

launch-rs-debug:
    {{editor}} tmp.rs
    cargo run --quiet -- show
    ls *.rs | entr -p sh -c 'export RUST_BACKTRACE=1; cargo run --quiet -- run --ignore-failures --command "cargo run --bin tmp"'

run:
    cargo run -- run --command "ruby tmp.rb"

# Test the stub generator with a random clash in ruby
test-stub-rb:
    cargo run -- next
    cargo run -- generate-stub "ruby"

# Test the stub generator with a random clash in rust
test-stub-rs:
    cargo run -- next
    cargo run -- generate-stub "rust"

# Test the stub generator with a random clash in C
test-stub-c:
    cargo run -- next
    cargo run -- generate-stub "c"