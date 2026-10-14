# RCA-04 — CI `Build (...)` failed on all 6 platform jobs: split files missing imports (`undefined: os`, `undefined: apperror`, `undefined: io`, `"bytes" imported and not used`)

Repo: `alimtvnetwork/gitmap-v28` (`cli/`, Go) · CI workflow step: `Build (...)`
(all 6 platform jobs in the matrix) · Breaking commit: `83939b8ae` (the
300-line-cap file-split commit) · Fix commit: `31806e142`

## Symptom

The CI `Build (...)` step failed on **all 6 platform jobs at once** with four
compile errors, all of them import bookkeeping, none of them logic:

```
cmdagent/agent_claim_collision.go:207:15: undefined: os
cmdinstall/install_archive_magic.go:55:15: undefined: apperror
cmdinstall/install_archive_magic.go:59:16: undefined: io
cmdinstall/install_archive_download.go:4:2: "bytes" imported and not used
```

Key reading skill for the dummy AI: when the same build failure appears on
every OS runner simultaneously, the bug is in the source tree, not in the
environment. And Go compile errors of the form `undefined: <pkg>` inside a
file that clearly *uses* `<pkg>` mean exactly one thing: the import line is
missing from that file's `import (...)` block. The fourth error is the mirror
image: an import line is present in a file that no longer *uses* it, and Go
refuses to compile any file with an unused import.

## Root cause

**One line:** the file-split commit moved functions into new files without
carrying all of their import dependencies along, and left a stale import in
the trimmed-down donor file.

Explanation, step by step, for zero-context readers:

1. Go compiles packages as a unit, but **every file must declare its own
   complete import list**. Moving a function from file A to file B does not
   move its imports; the author must copy the needed import lines by hand (or
   let an IDE/tool do it).
2. Commit `83939b8ae` split three over-long files to satisfy the house
   300-line cap. Three import lists drifted in the process:
   - `releaseFileClaimsBySubtask` (and its siblings) landed in the new
     `cli/cmdagent/agent_claim_collision.go`, which calls
     `fmt.Fprintf(os.Stderr, ...)` (see lines 208 and 213 today). The `"os"`
     import line stayed behind in the donor file → `undefined: os` at 207:15
     (the first `os.Stderr` use in the moved code).
   - `readArchiveHeaderBytes` and the magic-byte detectors landed in the new
     `cli/cmdinstall/install_archive_magic.go`, which calls `os.Open`,
     `io.ReadFull`/`io.EOF`/`io.ErrUnexpectedEOF`, and
     `apperror.WrapSimple(...)`. Only `"bytes"` and `"os"` were carried over
     → `undefined: apperror` at 55:15 and `undefined: io` at 59:16.
   - The donor `cli/cmdinstall/install_archive_download.go` lost the
     `bytes.Equal`/`bytes.HasPrefix` users (the magic-byte functions moved
     out) but kept `"bytes"` in its import block → Go's hard rule
     `"bytes" imported and not used` at 4:2.
3. Go treats a missing import as `undefined: <name>` rather than "you forgot
   an import" because the compiler only sees the file in front of it. That is
   why the error reads like the package does not exist.

The bug class is "mechanical refactor fallout": the code was correct before
the move and correct after, in every file, except the import declarations.
It is the most common failure mode of any manual file split, which is why the
prevention step below is a fixed ritual, not a suggestion.

## How it was found (exact commands)

All commands run from the gitmap-v28 working copy. `gitmap pe` is GitMap's
read-only "pull / examine" diagnostic that surfaces the failing tree state.

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/gitmap-v28

# 1. See the failing state (read-only diagnostics)
gitmap pe

# 2. Pull the exact failure out of the CI job log
#    (replace <build-job> with the failed Build job id from `gh run view`)
gh run view --job <build-job> --log-failed
#    -> prints the four `undefined:` / `imported and not used` lines quoted
#       under "Symptom"

# 3. Confirm each error by reading the import block and the flagged line
#    (no grep allowed in this house; gitmap cat/aum search only)
gitmap cat cli/cmdagent/agent_claim_collision.go | sed -n '1,15p'
#    -> import block has no "os", but the file uses os.Stderr at lines 208/213
gitmap cat cli/cmdinstall/install_archive_magic.go | sed -n '1,12p'
#    -> import block has "bytes" and "os" only; code uses io.* and apperror.*
gitmap cat cli/cmdinstall/install_archive_download.go | sed -n '1,18p'
#    -> import block still lists "bytes", but no bytes.* call remains in the file
gitmap aum search 'bytes\.' cli/cmdinstall/install_archive_download.go
#    -> 0 hits: the import is provably unused
```

Step 2 is the whole diagnosis: each error names `file:line:symbol`, so the
fix is a direct lookup — open the file, look at the import block, add or
remove the named line.

## Fix

Three files, all import-block-only changes, then `gofmt -w`. No logic touched.

**File 1 — `cli/cmdagent/agent_claim_collision.go` (added `"os"`):**

Before (import block, lines 3–12):

```go
import (
	"database/sql"
	"encoding/json"
	"fmt"
	"strings"
	"time"
	...
)
```

After (verified on disk today, line 7):

```go
import (
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"time"

	appfault "github.com/alimtvnetwork/gitmap-v28/cli/apperror"
	"github.com/alimtvnetwork/gitmap-v28/cli/store"
	"github.com/alimtvnetwork/gitmap-v28/cli/types"
)
```

**File 2 — `cli/cmdinstall/install_archive_magic.go` (added `"io"` and the
unaliased `apperror` import):**

Before:

```go
import (
	"bytes"
	"os"
)
```

After (verified on disk today, lines 3–9):

```go
import (
	"bytes"
	"io"
	"os"

	"github.com/alimtvnetwork/gitmap-v28/cli/apperror"
)
```

Note the unaliased `apperror` import: that is the `cmdinstall` package
convention — the sibling file `install_archive_download.go` imports
`"github.com/alimtvnetwork/gitmap-v28/cli/apperror"` unaliased too, while
`cmdagent` aliases it as `appfault`. Copy the convention of the *package you
are editing*, not the package you copied the code from.

**File 3 — `cli/cmdinstall/install_archive_download.go` (removed `"bytes"`):**

Before: the import block listed `"fmt"`, `"bytes"`, `"io"`, … After: the
`"bytes"` line is gone (verified on disk today — the block now runs
`"fmt"`, `"io"`, `"net/http"`, … with no `"bytes"`).

Commit facts: fix shipped as commit `31806e142` (import fixes + `gofmt -w`).

## Verification evidence

1. After the fix, the three commands the brief's prevention step names all
   came back clean: `go build ./...` (exit 0), `go vet ./...` (exit 0),
   `gofmt -l` (empty output — no files need formatting).
2. CI re-ran: the `Build (...)` step went green on all 6 platform jobs on the
   fix commit — a missing-import failure cannot regress at runtime once the
   tree compiles, because compilation *is* the test.
3. Today (2026-10-15) the three import blocks on disk still show the fixed
   state (verified via `gitmap cat` on each file, as shown under "How it was
   found"): `"os"` present in `agent_claim_collision.go`, `"io"` + unaliased
   `apperror` present in `install_archive_magic.go`, no `"bytes"` in
   `install_archive_download.go`.

## Prevention — checklist item

**Post-split compile ritual — run before every push that moves code between
files:**

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/gitmap-v28
go build ./... && go vet ./... && test -z "$(gofmt -l cli/)"
```

Plus the manual check no tool does for you: **diff the import blocks on both
sides of the split.** For the new file: every package name used in the moved
code must appear in its import block. For the donor file: every import must
still have at least one use left (`gitmap aum search '<pkg>\.' <donor-file>`
returning 0 hits means delete the import line).

One-line rule for the dummy AI: **a file split is not done when the code
moves — it is done when `go build ./...` passes.** Imports are per-file, not
per-package; the compiler will not carry them for you.
