# Subtask 02: Sequential E2E Verification and Minor Release

- **Status**: `COMPLETED`

## Sequential E2E Checklist
- [x] 1. Build frontend (`npm run build`) and local binaries (`agm`, `agm-alim`) with `-j 1`.
- [x] 2. Verify CLI prompt operations sequentially on `default` and `default-copy-5597`:
  - `agm prompts tree -i default` (cold scan `2402 ms` vs warm cache `183 ms`) and `agm prompts reset-tree -i default --json`.
  - `agm prompts running -i default` and `agm prompts running -i default --json`.
  - `agm prompts copy -i default --json` and `agm prompts resend 5 --json`.
  - `agm prompts backup -i default --json` and `agm prompts restore -i default --json` (`162 ms`, zero pipe hang).
  - `agm auto-switch status --json` and `agm auto-switch test 98` (zero pipe hang, threshold restored).
- [x] 3. Launch `agm-alim` GUI via CLI (`agm instances open-ui`), verify:
  - Update Available popup does not appear when no newer release with a Windows `.exe` asset exists.
  - Instances view renders both `Default` and `Default Copy` with Auto-Switch ON.
  - Prompt Tree modal opens immediately with `All Projects (48)` populated, thread count stays at `51–54` threads, and Reset / Copy buttons work.
  - Captured screenshots under `assets/screenshots/`.
- [x] 4. Run `npm run bump minor`, update `CHANGELOG.md`, `CHANGELOG_EN.md`, `readme.md`, and `README_EN.md` using relative paths only and `@aukgit` attribution, run pre-flight checks, commit, tag, and push.
