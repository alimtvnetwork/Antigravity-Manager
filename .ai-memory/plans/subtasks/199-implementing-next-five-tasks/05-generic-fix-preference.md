# Subtask 199-05: Generic-Fix Preference Guard

Status: IN PROGRESS (TAKEN)
Owner: Agent-199-Lead
Spec: `02-spec/21-app/199-implementing-next-five-tasks/spec.md` (Task 5)

## Target
- Create `src-tauri/src/modules/release_gate/generic_fix_preference_guard.rs`
- Create `src-tauri/src/modules/release_gate/tests_generic_fix_preference.rs`
- Do NOT edit `src-tauri/src/modules/release_gate/mod.rs` (lead wires registration).

## Work
Implement a hardcoding detector over described match sites. Input: `&[MatchSite]` where each site describes one matching construct in routing/sanitization-style code: kind (`ExactEqualityChain` with branch count, `LiteralLookupTable` with entry count, `WildcardPattern`, `RegexPattern`, `RegistryLookup`), the literal values involved, and an optional justification string. Output `GeneralizationReport`:
- flag exact-equality chains with >= 3 branches and literal tables with >= 4 entries as `GeneralizationRequired` unless a justification is present,
- wildcard/regex/registry sites pass,
- a single deliberate exact match with justification passes.
Each finding names the site and the suggested generalization (wildcard pattern, regex, or registry). Pure functions, no I/O, house guard style.

## Tests (in the tests file, house style)
- Five-branch exact-match chain without justification is flagged.
- Wildcard pattern matcher passes.
- Single exact match with justification passes.
- Literal table with many entries is flagged; with justification it is reported as acknowledged, not required.

## Invariants
Zero builds/tests during implementation. Relative paths only. GitMap-only search. No silent errors.
