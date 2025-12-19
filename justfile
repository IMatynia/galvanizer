fix_and_format:
    cargo clippy --fix --allow-dirty --all-targets
    cargo fmt