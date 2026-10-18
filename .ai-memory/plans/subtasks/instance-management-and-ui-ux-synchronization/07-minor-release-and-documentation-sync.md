# Subtask 07: Minor Version Bump & Atomic Release Ceremony

**Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`  
**Status:** `QUEUED`  
**Owner:** Lead Orchestrator  
**Exclusive Files:**  
- `package.json`  
- `src-tauri/tauri.conf.json`  
- `src-tauri/Cargo.toml`  
- `changelog.md`  
- `changelog_en.md`  
- `README.md`  
- `README_EN.md`  

## Objectives & Deliverables
1. Run pre-flight checks: `cd src-tauri && cargo fmt -- --check`, `cd src-tauri && cargo clippy --all-targets --all-features`, `npm run build`.
2. Execute `npm run bump minor` to atomically update 14 manifest locations.
3. Update `changelog.md` and `changelog_en.md` strictly attributing `@aukgit` (`(Thanks to @aukgit)`).
4. Synchronize release notes in `README.md` and `README_EN.md`.
5. Atomic commit and push via `gitmap cpf` or `gitmap cpb`.
