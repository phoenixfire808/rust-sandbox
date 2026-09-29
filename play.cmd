@echo off
setlocal
cd /d "%~dp0"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=D:\jcode-build\rust-sandbox-20260929"
if not exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
    echo Rust Cargo was not found. Install Rust or run the built source-map.exe.
    if not defined JCODE_NONINTERACTIVE pause
    exit /b 1
)
echo Loading original locally installed Garry's Mod content into Bevy...
"%USERPROFILE%\.cargo\bin\cargo.exe" run --locked -p rust-sandbox --bin source-map -- %*
set "RESULT=%errorlevel%"
if not "%RESULT%"=="0" (
    echo.
    echo Launch failed. The error is shown above. No Steam files were changed.
    if not defined JCODE_NONINTERACTIVE pause
)
exit /b %RESULT%
