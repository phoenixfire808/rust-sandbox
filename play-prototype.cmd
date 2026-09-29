@echo off
setlocal
cd /d "%~dp0"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=D:\jcode-build\rust-sandbox-20260929"
"%USERPROFILE%\.cargo\bin\cargo.exe" run --locked -p rust-sandbox --bin rust-sandbox -- %*
exit /b %errorlevel%
