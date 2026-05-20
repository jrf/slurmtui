default:
    @just --list

build:
    cargo build

release:
    cargo build --release

install: release
    cp target/release/slurmtop ~/.cargo/bin/

run:
    cargo run

clean:
    cargo clean

check:
    cargo check
