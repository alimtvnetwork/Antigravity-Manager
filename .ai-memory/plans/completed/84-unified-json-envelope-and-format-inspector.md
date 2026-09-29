# Master Execution Plan: Unified JSON Envelope Schema & Smart Format Inspector

> **/goal** Implement system-wide `{ "attributes": ..., "data": ... }` JSON envelope standards, smart format inspector `agm which-format`, mutation previews, bulk import generators, and migrate `d:\work\repo-secrets`.

## User Request (Verbatim)
```text
/goal I think we have different types of JSON, right? Import, export, deploy, many things. So what we should do is JSON will have a type. Okay? Output as a type. And when we output JSON or any data type, we will have two distinct sections on top. First is the attributes, which actually contains the data type, where it is coming from, how it is coming from. The other is the data. The data contains most of the information in the node. So we should have a command in the CLI that would actually tell us, if we do the CLI space, which format, and then pass the JSONs. Or if we don't pass any JSON and folder has multiple JSONs, it will automatically take the JSONs as a format. It will understand and share a command that we can use to run these commands to import. And what will be the end result, what it will change, it will give a summary as well. So let's say a folder contains five JSON. Okay? Three of the JSON really matches the format that we have in the system. So it will tell us, these are the three. If you want to import each one of them, this is the command you should type, and you will get it, and this will change, these things. Now, the last two does not match, so it will say, these are the two things we could not match, so we will not import it. And we will also provide a single line of command that user can use to import all of these. Okay? And if they have to prompt something, they could also say, or the suggestion also could say, you could do "-y" to bypass all the prompting to yes. Okay? So this is one of the techniques that I want in all over the code base. I want to make a huge change. So please make detailed planning so that we don't miss anything. We have a consistency. And according to this, I also want you to go inside the repo secrets and change all the JSONs that we have to this order and this format. Do you understand? Do you have any question and confusion? If you have, let me know. Other than that, please start working on it. And at the end, you verify all these types that we have. Do the end-to-end testing here as much as possible. Okay? Okay. And add some samples. Do not pass any security-based data like password or anything to CI/CD or commit this data. Remember that. So any secret things, it should be inside the repo secrets folder, nothing outside. Remember that. Very important. Can you please apply these changes as I've mentioned? Is it clear?
```

## Traceable Subtasks & Execution Results
- **Subtask 01**: Canonical Spec authoring (`02-spec/21-app/02-unified-json-envelope-and-format-inspector-spec.md`) -> [DONE]
- **Subtask 02**: Rust Module `src-tauri/src/modules/json_envelope.rs`: Envelope structs, format classifier, signature heuristics, and mutation diff summary -> [DONE]
- **Subtask 03**: CLI Command `agm which-format` (and alias `agm format inspect`) in `src-tauri/src/bin/agm.rs` -> [DONE]
- **Subtask 04**: CLI Importers integration (`cmd_supabase_load_json`, accounts, instances) supporting unwrapped & wrapped JSON with `-y` -> [DONE]
- **Subtask 05**: Migrate existing JSON files in `d:\work\repo-secrets` to `{ "attributes": ..., "data": ... }` schema; commit and push in `repo-secrets` -> [DONE]
- **Subtask 06**: Create sanitized dummy samples in `tests/fixtures/json_envelope/` (NO real secrets in main repo) -> [DONE]
- **Subtask 07**: Unit and integration test verification, pre-flight cargo fmt + clippy check, frontend build gate -> [DONE]
- **Subtask 08**: Update JSON exporters (`agm accounts --json`, `agm instances --json`) to emit standardized `{ "attributes": ..., "data": ... }` envelopes -> [DONE]

## Verification Log
1. **Unit Tests**:
   - `modules::json_envelope::tests::test_bulk_command_generation` -> PASSED
   - `modules::json_envelope::tests::test_unrecognized_schema_rejection` -> PASSED
   - `modules::json_envelope::tests::test_legacy_format_detection` -> PASSED
   - `modules::json_envelope::tests::test_envelope_serialization_and_unpacking` -> PASSED
2. **Formatting & Linter Gates**:
   - `cargo fmt -- --check`: Clean (0 diffs)
   - `cargo clippy --bin agm`: Clean (0 warnings in bin `agm` or `json_envelope`)
   - `npm run build`: Clean (built in 15.86s, 0 errors)
3. **End-to-End CLI Verification**:
   - `target/debug/agm which-format ..\tests\fixtures\json_envelope` verified recursive discovery, schema matching, mutation previews, rejection of unmatched files, and single-line bulk command generation with `-y`.
   - `target/debug/agm accounts --json` verified envelope emission with `attributes.type = "agm/accounts-export"`.
   - `target/debug/agm instances --json` verified envelope emission with `attributes.type = "agm/instances-export"`.
4. **Secret Storage Isolation**:
   - 10 real secret configurations safely migrated and pushed exclusively in `repo-secrets` (`1eb9822`).
   - Only sanitized mock files (`tests/fixtures/json_envelope/*`) stored in the public repo.
