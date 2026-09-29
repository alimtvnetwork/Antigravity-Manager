# Manual Guide for Closing Integrated Pull Requests

If you prefer not to use the GitHub CLI, follow these steps to manually close integrated pull requests.

## Integrated PR List

The following pull requests have been manually integrated into the project:

1. **PR #825** - [Internationalization] Device Fingerprint Dialog localization (@IamAshrafee)
2. **PR #822** - [Japanese] Add missing translations and refine terminology (@Koshikai)
3. **PR #798** - [Translation Fix] Correct spelling error in Vietnamese settings (@vietnhatthai)

---

## Instructions

For each pull request, follow these steps:

### 1. Open the PR Page

Navigate to the corresponding pull request:

- https://github.com/alimtvnetwork/Antigravity-Manager/pull/825
- https://github.com/alimtvnetwork/Antigravity-Manager/pull/822
- https://github.com/alimtvnetwork/Antigravity-Manager/pull/798

### 2. Post Appreciation Comment

In the comment box at the bottom of the pull request page, paste the following note:

```markdown
Thank you for your contribution! 🎉

The changes from this PR have been integrated into the project codebase.

The updates are documented in:
- README.md changelog
- Contributors list

Thank you again for your support of the Antigravity Manager Tools project!
```

### 3. Close the Pull Request

1. Click the **"Close pull request"** button beneath the comment field.
2. Alternatively, click **"Close with comment"** to submit the thank-you note and close the PR simultaneously.

---

## GitHub CLI One-Liner

If you have GitHub CLI installed (`gh`), close all three PRs with comments in a single command:

```bash
for pr in 825 822 798; do
  gh pr comment $pr --body "Thank you for your contribution! 🎉 This PR has been manually integrated."
  gh pr close $pr
done
```
