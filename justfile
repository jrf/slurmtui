default:
    @just --list

build:
    cargo build

release:
    cargo build --release

install: release
    cp target/release/slurmtui ~/.cargo/bin/

run:
    cargo run

clean:
    cargo clean

check:
    cargo check
