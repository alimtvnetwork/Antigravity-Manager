---
name: fix-repo URL-handling rule
description: fix-repo.ps1/.sh replace the versioned-repo-name token in ALL text files including inside URLs — host stays untouched, only the repo-name path segment changes.
type: feature
---

# fix-repo URL handling

The `fix-repo` scripts (PowerShell + Bash, repo-root) treat the
versioned repo-name token as a plain substring. They replace EVERY
occurrence in EVERY tracked text file — including inside URLs.

**Why:** The host (`github.com`, `gitlab.com`, …) is never touched
because the host string is not the token. Only the path segment that
literally equals `{RepoBase}-v{N}` is rewritten to
`{RepoBase}-v{Current}`. So `https://github.com/owner/coding-guidelines-v24/blob/...`
becomes `https://github.com/owner/coding-guidelines-v24/blob/...` —
the host and the rest of the path are preserved automatically.

**How to apply:** No URL detection, no `https?://` skipping, no
allow-list / deny-list of file types beyond text-vs-binary. Both
default mode (last 2 versions) and `-all` mode behave identically
with respect to URLs. This rule was confirmed by the user in the
fix-repo design review (Apr 2026, "all text files regardless of
url or not, remember it").
