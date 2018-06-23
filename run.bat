REM stop cargo from destroying your windows console

SET RUST_LOG=
cargo build
SET RUST_LOG=debug
target\debug\rust-music-bot-futures.exe
set RUST_LOG=