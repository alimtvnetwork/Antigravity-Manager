@echo off
setlocal
set "SCRIPT_DIR=%~dp0"
set "AGM_RELEASE=%SCRIPT_DIR%src-tauri\target\release\agm.exe"
set "AGM_DEBUG=%SCRIPT_DIR%src-tauri\target\debug\agm.exe"

if exist "%AGM_RELEASE%" (
    "%AGM_RELEASE%" %*
) else if exist "%AGM_DEBUG%" (
    "%AGM_DEBUG%" %*
) else (
    where agm >nul 2>&1
    if %ERRORLEVEL% equ 0 (
        agm %*
    ) else (
        cargo run --manifest-path "%SCRIPT_DIR%src-tauri\Cargo.toml" --bin agm -- %*
    )
)
