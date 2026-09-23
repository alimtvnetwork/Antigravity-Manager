# Specification 21: PR #4 Upstream Sync Merge, Architecture Protection & Documentation Localization

## 1. Architectural Scope & Purpose

This specification governs the safe assimilation of upstream PR #4 (`lbjlaq:main` -> `main` in `alimtvnetwork/Antigravity-Manager`), guaranteeing the preservation of our superior enterprise build system, version release lifecycle, cross-platform installers, and branding, while integrating valuable upstream features and converting all upstream documentation and release utilities from Chinese to English.

---

## 2. User Request (Verbatim)

```text
https://github.com/alimtvnetwork/Antigravity-Manager/pull/4

Hi there. Try to merge this PR. After merging, remember to keep our codes intact, okay? Keep our version release process intact. So everything that you merge, change back the codes that actually reverse and also harms in our code, okay? Can you do that and make sure that things are working fine after the merge? Do you understand the task? Can you please help me with this? Especially the build sections, our build is far more superior, right? And also in the future, the changes that we do, try to keep this logically outside so that we do not collide with the existing files, okay? We write new codes, new package, and try to make it work, okay? For example, some files, let's say, release guide, it has full Chinese writing, right? So these are the files I want you to convert to fully English, okay? After you merge it, remember this. So all these docs or release guide, those are in, let's say, Chinese. Convert those to English after you merge and resolve the... They do also have VersionBun. I'm really surprised. Okay. We can have the VersionBun. All right. They have created it too. So the language, it should be English as we have done it. So in most cases, our code will be superior. So the changes that they have done, we also wanted to keep it, okay? But also at the same time, we put the codes that is done by us and has high priority. Easy, clear?
```

---

## 3. High-Priority Architectural Invariants (Must Remain 100% Intact)

1. **Version Source of Truth & Release Process**:
   - `version.json` remains the single source of truth at version `4.65.0`. Upstream attempts to regress to `4.8.0` or `4.7.13` MUST be rejected.
   - Release orchestrator (`03-ai-scripts/29-release-orchestrator.py`) and sync checkers remain the canonical release mechanism.
2. **Branding & Identity**:
   - Title: `Antigravity Manager Tools`.
   - Publisher / Subtitle: `Maintained by Alim, Sponsored by RISEUP ASIA LLC`.
   - Windows Registry hooks (`src-tauri/hooks.nsh`), `tauri.conf.json`, `package.json`, and Cargo manifests preserve this branding.
3. **Build Architecture & Packaging Hooks**:
   - Universal macOS lipo bundling hook (`scripts/before-bundle.js` and `beforeBundleCommand`).
   - Default run binary (`default-run = "agm-alim"` in `src-tauri/Cargo.toml`).
   - NSIS hooks (`src-tauri/hooks.nsh`).
   - Standalone installers (`install.ps1`, `install.sh`) with 10-version fallback ladder, quiet aria2c delegation, and asset decoupling.
4. **Bottom Bar Reactive Update Mechanism**:
   - Centralized update store (`src/stores/use-update-store.ts`).
   - Desktop pagination center slot (`src/components/common/Pagination.tsx`).
   - Interactive update pill in `src/pages/Accounts.tsx`.

---

## 4. Upstream Assimilation & Feature Retention Matrix

| Feature Domain | Upstream Contribution (`lbjlaq:main`) | Integration Strategy |
| :--- | :--- | :--- |
| **APIKEY.FUN Key Profiles** | Multiple key profiles, OpenCode sync (`opencode_sync.rs`, `ApiKeyFun.tsx`, `opencodeProfiles.ts`) | Adopt upstream logic into modular files; keep our styling intact |
| **Language Server Hot-Switch** | Restart only `language_server` child process without killing main IDE window (`integration.rs`, `process.rs`) | Adopt upstream logic; preserve our `#[allow(async_fn_in_trait)]` and universal bundle hooks |
| **Prompt Sanitizer & WAF Guard** | PromptSanitizer pipeline node, WAF pseudo-rate-limit cleansing (`prompt_sanitizer.rs`, `inbound.rs`) | Merge upstream pipeline improvements; ensure clean compile |
| **Lightweight Mode** | Resource-saving lightweight mode (`lightweight.rs`) | Adopt module cleanly |
| **UI Enhancements** | Drag-resizable table columns, request header copy in `ProxyMonitor.tsx` | Adopt UI improvements while preserving design system |
| **Documentation & Release Guide** | Chinese release guide (`docs/RELEASE_GUIDE.md`) | Translate 100% to English, lowercase filename `docs/release-guide.md` |
| **Release Bump Tool** | Node script `scripts/bump-version.mjs` in Chinese | Translate CLI prompts and docs to English; keep as secondary/companion utility |

---

## 5. Verification Gates

1. **Conflict-Free Merge**: Git tree merged cleanly with zero unresolved conflict markers.
2. **Zero TypeScript Errors**: `npx tsc --noEmit` exits 0.
3. **Zero Rust Compilation Errors**: `cargo check` in `src-tauri/` exits 0.
4. **No Regression on Branding / Versioning**: Verified `version.json`, `package.json`, `Cargo.toml`, `tauri.conf.json` are at `4.65.0` with proper branding.
5. **English Documentation Gate**: `docs/release-guide.md` and related scripts contain zero Chinese characters in prose/CLI prompts.
