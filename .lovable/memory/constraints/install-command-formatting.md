---
name: Install Command Formatting
description: One-line install commands only — no inline comments inside install code blocks; platform indicated by a header (### Windows / ### macOS · Linux) directly above the block. Match the on-site InstallSection UI (4 install commands + 7 named bundles) order exactly.
type: constraint
---

# Install Command Formatting

## The rule

**Every install command is exactly one line. Inside install code fences there is NEVER a `#` comment, NEVER a blank line, NEVER a multi-line `\` continuation.** The platform is communicated by a markdown header (`### Windows · PowerShell`, `### macOS · Linux · Bash`) placed **directly above** the fenced block — never inside it.

## Why

- A user copies a one-liner with one click. Any `# comment` line above the command makes them paste the comment too, which their shell either errors on or silently ignores — both are bad UX.
- The on-site `InstallSection` (`src/components/landing/InstallSection.tsx`) renders each command as a single `<code>` line with a `CopyButton`. The README must mirror that exact shape so docs and UI agree.
- The user has corrected this multiple times. Do not regress.

## ❌ FORBIDDEN

````markdown
```bash

# One-shot installers (no clone required)

#   PowerShell:

iwr https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/main/install.ps1 | iex

#   Bash:

curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/main/install.sh | bash
```
````

Reasons it is forbidden: comments inside the block, two platforms in one block, mixed shells, no per-platform header.

## ✅ REQUIRED

````markdown

### 🪟 Windows · PowerShell

```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/main/install.ps1 | iex
```

### 🐧 macOS · Linux · Bash

```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/main/install.sh | bash
```
````

## Placement in the README

1. The install section is the **first content block** after the badges and stats line — **before** the Table of Contents anchors any other content.
2. Order must match the on-site UI (`InstallSection.tsx`) at the top of the README:
   1. **Full repo install** (4 commands): Windows · Windows skip-probe · Bash · Bash skip-probe.
   2. **Named bundle installers** (7): `error-manage`, `splitdb`, `slides`, `linters`, `cli`, `wp`, `consolidated` — each with one Windows + one Bash line.
3. The **Bundle Installers** section must appear immediately after **Install in One Line** and before the Table of Contents.
4. The `Run Commands` developer section (`npm run …`) comes **after** the install section, never before it.

## How to apply

- When editing `readme.md`, audit every fenced block whose content begins with `irm`, `iwr`, `curl`, `wget`, or `npm install -g` and ensure it contains exactly one command and zero comment lines.
- The `scripts/fix-readme-code-blocks.mjs` formatter must not strip platform headers; install blocks are exempt from blank-line-before-return rules because they contain no control flow.
- If a future request asks to "add a note" inside an install block, refuse and put the note in prose **above** the block instead.
