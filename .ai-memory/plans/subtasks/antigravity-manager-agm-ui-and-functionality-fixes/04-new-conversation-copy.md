# Task-04: New conversation + copy-with-images

## Status
PENDING (needs Task-01 gap report)

## Objective
Add the "new conversation" option and verify/complete copy-with-images.

## Requirements
- Add a "new conversation" action in the prompt-tree action area
  (currently missing). It must create a fresh conversation in the target
  project/instance and select it.
- Copy: verify the copy path (`clipboard.ts` + actions) includes images in
  compact form. If it doesn't, fix it. In the final report, **explain how
  the copy works**: format, how images are embedded/compressed, what
  "compact" means concretely.
- Works for all projects, not just the active one.

## Acceptance
- [ ] New-conversation option present and functional.
- [ ] Copy verified to include images compactly (or fixed).
- [ ] Explanation of the copy mechanism in the final report.

## Constraints
- GitMap search only. Relative paths only.
- Coordinate with Task-02 on UI files (lead arbitrates).
