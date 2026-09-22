## Quick Install v4.59.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.59.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.59.0/install.sh | bash
```

---

## What's Changed in v4.59.0

- **Fix CI Concurrency Race (RCA-32)**: Synchronized `ABV_DATA_DIR` across unit tests using `TEST_DATA_DIR_MUTEX` in `config.rs` and `account.rs`, resolving multi-threaded data directory collision between parallel test runners.
- **Fix Windows CI Entrypoint (0xc0000139)**: Re-embedded `windows-test.manifest` for Windows MSVC test builds to link Common-Controls 6.0 as mandated by Rule 5 in instance management spec.
- **Clean Compiler Warnings**: Resolved unused imports across `models/mod.rs`, `modules/integration.rs`, `modules/repo_db.rs`, `commands/patch.rs`, `commands/supabase.rs`, `modules/cloudflared.rs`, and `proxy/server.rs`.
- **Platform-Specific Imports**: Added `#[cfg(target_os = "windows")]` to `CommandExtWrapper` in `email_inbound.rs` so that Windows-only process creation flags compile without triggering unused import warnings on Unix.
- **Verification**: 100% green verified across all 752 unit tests, `cargo fmt`, `cargo clippy`, TypeScript checks (`tsc --noEmit`), and Vite production builds.
