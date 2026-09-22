# Distribution

> **Version:** 1.0.0
> **Updated:** 2026-04-19

The linter pack ships **two ways**, both produced by the same release job
in `.github/workflows/release.yml`.

---

## 1. Versioned ZIP (universal)

Built into every GitHub Release as
`coding-guidelines-linters-vX.Y.Z.zip`. Contains:

```
linters-cicd/
├── checks/
├── ci/
├── configs/
├── action.yml
├── run-all.sh
├── install.sh
├── readme.md
└── VERSION
```

Install one-liner (Linux / macOS):

```bash
curl -fsSL https://github.com/alimtvnetwork/coding-guidelines-v24/releases/latest/download/install.sh | bash
```

The installer:

1. Downloads the matching `coding-guidelines-linters-<latest>.zip`
2. Extracts to `./linters-cicd/`
3. Verifies SHA-256 against `checksums.txt`
4. Prints next-step commands

Flags:

- `-d <dir>` install destination (default: `./linters-cicd`)
- `-v <version>` install a specific version (default: latest)
- `-n` skip checksum verification (not recommended)

---

## 2. GitHub composite Action

`linters-cicd/action.yml` lets GitHub users skip the install entirely:

```yaml
- uses: alimtvnetwork/coding-guidelines-v24/linters-cicd@v3.9.0
```

GitHub clones the repo at the specified ref and runs `action.yml`. Zero
maintenance for consumers — just bump the version pin.

---

## 3. Release Binary Asset Decoupling & Presence Assertion

1. **Installer Script Decoupling:** Standalone installer scripts (`install.ps1`, `install.sh`) MUST NOT be packaged into release assets. They are hosted at the repository root and fetched dynamically via git tag URLs.
2. **Mandatory Binary Asset Gate:** Every release workflow MUST assert that at least one executable binary or installer package (`*.exe`, `*.dmg`, `*.AppImage`, `*.deb`, `*.rpm`, `*.zip`) is present in `release-files/` before creating or updating a release. Publishing an empty release is strictly prohibited.

---

## 4. Release Notes Code Block Standard

Every published release MUST format quick-install commands into separate, dedicated Markdown code blocks for Windows and POSIX, with 1-click copy support:

- **Windows Latest:** `irm https://raw.githubusercontent.com/<owner>/<repo>/main/install.ps1 | iex`
- **Windows Pinned:** `irm https://raw.githubusercontent.com/<owner>/<repo>/<tag>/install.ps1 | iex`
- **Linux/macOS Latest:** `curl -fsSL https://raw.githubusercontent.com/<owner>/<repo>/main/install.sh | bash`
- **Linux/macOS Pinned:** `curl -fsSL https://raw.githubusercontent.com/<owner>/<repo>/<tag>/install.sh | bash`

---

*Part of [CI/CD Integration](./01-index.md)*
