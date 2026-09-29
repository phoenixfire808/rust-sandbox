@echo off
setlocal
cd /d "%~dp0"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=D:\jcode-build\rust-sandbox-20260929"
cargo run --locked -p rust-sandbox -- %*
exit /b %errorlevel%
