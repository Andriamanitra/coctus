# Check outdated formatting
outdated:
    cargo run --quiet -- show 25623694f80d8f747b3fa474a33a9920335ce | less -R
    cargo run --quiet -- show 7018d709bf39dcccec4ed9f97fb18105f64c  | less -R

# Check Monospace
mono:
    cargo run --quiet -- show 1783dda5b69105636695dc5bf51de1baf5d0  | less -R
    cargo run --quiet -- show 1222536cec20519e1a630ecc8ada367dd708b | less -R
    cargo run --quiet -- show 6357b99de3f556ffd3edff4a4d5995c924bb  | less -R
    cargo run --quiet -- show 4730251b4f27c2549cffc6fa48c40b7b85c8  | less -R

# Check tags nesting
nested:
    cargo run --quiet -- show 750741cba87bb6a6ac8daf5adbe2aa083e24  | less -R
    cargo run --quiet -- show 83316b323da5dba40730dbca5c72b46ccfc9  | less -R

# Check self nesting
nested-self:
    cargo run --quiet -- show 70888dd5bb12f2becdad5e6db3de8b40a77f  | less -R

# Check not matching tags
not-matching:
    cargo run --quiet -- show 7040402a6fe461068f5cf5296607c184d043a | less -R