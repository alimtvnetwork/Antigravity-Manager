# Subtask 06: End-to-End Testing Verification, Coding Guidelines Audit & Minor Bump Release

- **Slug**: `e2e-testing-and-minor-bump-release`
- **Component**: Testing, Audit & Release Ceremony
- **Primary Files**:
  - `package.json`
  - `version.json`
  - `changelog.md`
  - `changelog_en.md`
  - `README.md`
  - `README_EN.md`
- **Status**: QUEUED

---

## 1. Problem Statement
The user explicitly stated:
> "Make sure at the end you do a minor bump and release after doing the end-to-end testing on these three features, as I have mentioned. Also, how much coding guideline is followed? Can you please give me a percentage right now? How much we should improve as well, based on the current code base, how much the improvements areas are. Please focus on this."

The three primary features to verify via E2E:
1. **Send Preemptive Cancellation vs Enqueue**: Send cancels active task and runs immediate prompt; Enqueue queues behind active task without interrupting.
2. **IDE Conversation Title Renaming Sync**: Renaming in AGM updates both AGM and IDE state.
3. **Resizable Left-Hand Prompt Tree Panel**: Tree view width can be resized smoothly and saved.

## 2. Technical Requirements
1. **Targeted End-to-End Testing**:
   - Write and execute targeted E2E / integration verification script proving:
     a. Send triggers preemptive cancellation and immediate dispatch.
     b. Enqueue preserves running task and appends to FIFO queue.
     c. Renaming title updates IDE workspace metadata.
     d. Resizable tree adjusts width and respects bounds.
2. **Coding Guidelines Adherence Report**:
   - Deliver full percentage analysis and itemized breakdown of compliance and recommendations.
3. **Minor Version Bump**:
   - Synchronize version manifests (`npm run bump minor`).
   - Update `changelog.md` and `changelog_en.md` strictly attributing `@aukgit` (`(Thanks to @aukgit)`).
   - Synchronize `README.md` and `README_EN.md` release sections.
   - Run pre-flight checks: `cargo fmt -- --check`, `cargo clippy`, `npm run build`.
   - Commit via GitMap hyphen-separated atomic commit (`gitmap cpf "<module> - <summary>"`).

## 3. Verification Criteria
- All tests pass.
- Version is cleanly bumped to next minor.
- Repository is clean and pushed.
