# Completed Plan: accounts-table-delete-hover-right-click-disable

- **Task Slug:** `accounts-table-delete-hover-right-click-disable`
- **Protocol:** `@[.agents/skills/execute-parent-task-with-n-steps-v6]`
- **Budget:** N = 300 steps (Phase 1: 150, Phase 2: 150)
- **Concurrency Capacity:** A = 2, H = 2 (Workers 01 and 02)
- **Task Database:** `.ai-memory/temp-agents/09-accounts-table-delete-hover-right-click/agent-task.db`
- **Audit Ledger:** `.ai-memory/temp-agents/09-accounts-table-delete-hover-right-click/ledger.md`
- **Target Release:** Minor Version Bump (`v4.221.0`)
- **Status:** `COMPLETED` (100% verified)

---

## 1. User Request (Verbatim)

```markdown
# High Priority Instruction

The Delete button does not work yet, and I have given several times. It also needs to be fixed by selecting. If we select and click on Delete, it does not work as well. Make sure that you fix it. And when we hover over to an item, try to make that line as white and make the text as black. Invert the colors, but do not make those bold or anything, then the position will change. It looks bad. Also, make sure when we hover, the buttons should be highlighted right inside. The button role needs to be changed as well. And also, the switch button needs to be in a different yellow highlighter color, which should be different than others. Also, the Remove button should be red and white. The background would be red. The color would be white. Also add a disable option. So if we disable any account by selecting or by right-clicking on it, that disable item would not be selected by passport or the switching automatically. By default, user can try to switch. That's a different story. If you understood, then implement this and bump the minor version, and then release.

# Actionable Items Must Follow Non-Negotiable

0. Gitmap do `gitmap pull <repo path> or run into the repo path` Make sure we have the latest and conflict resolve before starting.
1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Fix the Delete button functionality to work both by clicking and selecting.
5. Implement hover effect to change line color to white and text to black without altering position.
6. Ensure buttons are highlighted correctly on hover.
7. Change button roles as specified.
8. Set switch button to a unique yellow highlighter color.
9. Style Remove button with red background and white text.
10. Add disable option for accounts, ensuring disabled items are not selectable or switchable.
11. Bump the minor version and prepare for release.
```

---

## 2. Ingested Evidence

- **Reference Screenshot:** `assets/screenshots/accounts-table-delete-hover-right-click-disable-01.png`

---

## 3. Disjoint Ownership Matrix & Delivered Outcomes

| Worker | Subtasks | Strictly Owned Files | Delivered Outcome |
| :--- | :--- | :--- | :--- |
| **Worker 01** | `Task-02` (Subtask 1) | `src-tauri/src/modules/account/crud.rs`<br>`src/pages/accounts/useAccountsHandlers.ts`<br>`src/pages/accounts/AccountsDialogs.tsx`<br>`src/components/accounts/AccountTable/AccountRowContextMenu.tsx`<br>`src/components/accounts/AccountTable/SortableAccountRow.tsx` | - Implemented 3-iteration transient file-lock retry ladder (25ms, 50ms, 100ms) with `record_ignored` fallback in `crud.rs`, ensuring `trigger_account_delete` unconditionally fires.<br>- Restructured `useAccountsHandlers.ts` with structured `finally` blocks so delete modals never get stuck open.<br>- Created `AccountRowContextMenu.tsx` rendered via `createPortal` with viewport edge clamping, outside click/Escape dismissal, and action dispatchers.<br>- Mounted context menu on `SortableAccountRow.tsx` via `onContextMenu`. |
| **Worker 02** | `Task-03`, `Task-04` (Subtasks 2 & 3) | `src/components/accounts/AccountTable/SortableAccountRow.tsx`<br>`src/components/accounts/AccountTable/AccountRowContent.tsx`<br>`src/components/accounts/AccountTable/cells/ActionsCell.tsx`<br>`src/components/accounts/AccountCard/AccountCardActions.tsx`<br>`src/components/accounts/AccountRow.tsx` | - Implemented unconditional `group-hover:!text-black dark:group-hover:!text-black` across all rows (including active, selected, and focused states), permanently eliminating amber-on-white or cyan-on-white contrast failures.<br>- Permanently locked typography weight to fixed `font-medium` (eliminated all dynamic `font-bold` and `font-semibold` shifts) and static `border-l-2`, preventing layout jitter.<br>- Styled Switch button with high-contrast yellow highlighter theme (`bg-yellow-400 text-slate-900 border-yellow-500 hover:bg-yellow-300`).<br>- Styled Remove button with solid red background and white icon/text (`bg-red-600 text-white border-red-700 hover:bg-red-700`) without blocking deletion of disabled accounts.<br>- Enforced semantic `role="button"`, `type="button"`, and explicit `aria-label` across all action buttons. |
| **Lead** | `Task-01`, `Task-05` | `package.json`, `package-lock.json`, `version.json`, `releases-manifest.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/hooks.nsh`, `Casks/antigravity-tools.rb`, `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, `README_EN.md` | - Pre-flight quality gates (`fmt`, `clippy`, `cargo test`, `npm run build`, `secrets gate`).<br>- Atomic version bump to `v4.221.0`.<br>- GitMap atomic commit (`gitmap cpf`) and git tag push. |

---

## 4. Verification Evidence

- `cargo fmt -- --check`: PASS (exit code 0)
- `cargo clippy --all-targets --all-features`: PASS (exit code 0)
- `cargo test --lib modules::account`: PASS (exit code 0, 15/15 tests passed)
- `npm run build`: PASS (exit code 0, built in 15.49s)
- `gitmap aum search` Secrets Gate: PASS (0 hits across 931 files)
