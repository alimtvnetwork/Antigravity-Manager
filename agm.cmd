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
    for /f "delims=" %%I in ('where.exe agm.exe 2^>nul') do (
        if exist "%%I" (
            "%%I" %*
            exit /b %ERRORLEVEL%
        )
    )
    cargo run --manifest-path "%SCRIPT_DIR%src-tauri\Cargo.toml" --bin agm -- %*
)
