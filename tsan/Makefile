.PHONY: all

all:
        RUSTFLAGS="-Zsanitizer=thread" cargo test -Zbuild-std --target x86_64-unknown-linux-gnu 2>&1
