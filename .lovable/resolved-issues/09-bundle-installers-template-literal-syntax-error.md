# RCA: JavaScript Template Literal SyntaxError in `generate-bundle-installers.mjs`

- **Date:** 2026-08-30
- **Status:** Resolved
- **Severity:** Blocker (CI/CD Pipeline Build Failure)
- **Target File:** `scripts/generate-bundle-installers.mjs`

---

## 1. Why It Happened
During CI/CD execution of `node scripts/generate-bundle-installers.mjs`, Node.js failed to parse the ES module due to `SyntaxError: Unexpected identifier 'n'`.

## 2. How It Happened
The script generates PowerShell and Bash installer scripts using multiline JavaScript template literals (enclosed in backticks `` ` ``). Inside the embedded PowerShell script block:
```powershell
Add-Content -Path $dstFile -Value "`n`n### [Auto-Merged from Coding Guidelines Update]" -Encoding UTF8
```
The PowerShell backtick-n (`` `n ``) was written without escaping the backtick inside the JavaScript template string. As a result, Node.js interpreted the first backtick as the closing delimiter of the JS template string, followed by an unrecognized bare identifier `n`.
Similarly, embedded Bash variable expansions `${src}` and `${TARGET}` in lines 406 and 432 were evaluated by JavaScript as template expressions rather than escaped bash variables `\${src}`.

## 3. Root Cause
PowerShell newline escape characters (`` `n ``) and Bash variable interpolations (`${var}`) were not escaped with backslashes (`\`n`, `\${var}`) inside multiline JavaScript template strings.

## 4. Code Fix & Prevention Rule
1. **Fix Applied:** Escaped all PowerShell backticks (`\`n\`n`) and Bash template expressions (`\${src}`, `\${TARGET}`, `\${dest}`) in `scripts/generate-bundle-installers.mjs`.
2. **Local CI Verification:** Registered `node scripts/generate-bundle-installers.mjs` into `03-ai-scripts/03-cicd-local-runner.py` under the `Bundle Installer Generation` quality gate.
3. **Prevention Rule:** Any shell script embedded inside a JavaScript template literal MUST strictly escape all backticks (`` \` ``) and dollar-bracket interpolations (`\${...}`).
