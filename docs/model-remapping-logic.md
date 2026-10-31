# Model remapping logic (current implementation)

Last updated: 2026-03-02

This article describes the model remapping link in the current agent (with adjusted behavior for Gemini 3/3.1 Pro).

## 1) Overall process

Whether it is the OpenAI protocol or the Gemini native protocol, the request model will go through two stages of processing:

1. Static route resolution (global rules):
   - Execute this once before selecting an account.
   - Code: `resolve_model_route` of `src-tauri/src/proxy/common/model_mapping.rs`.
2. Dynamic account awareness rewriting (conditional fallback):
   - Execute after selecting the account.
   - Code: `resolve_dynamic_model_for_account` in `src-tauri/src/proxy/token_manager.rs`.

Use the entry point of this process:
- `src-tauri/src/proxy/handlers/openai.rs`
- `src-tauri/src/proxy/handlers/gemini.rs`

## 2) Static routing priority

The priority of `resolve_model_route(original_model, custom_mapping)` from high to low is:

1. Official dynamic elimination and forwarding rules:
   - `DYNAMIC_MODEL_FORWARDING_RULES`
2. User-defined precise mapping:
   - `custom_mapping[original_model]`
3. User-defined wildcard mapping:
   - Compare by "number of non-`*` characters", the more specific the higher the priority
4. System built-in default mapping:
   - `map_claude_model_to_gemini`

If neither hits, the model name will be passed through as it is.

## 3) Current Gemini Pro built-in mapping strategy

The current strategy is: the specific model ID is directly transmitted transparently; only the generic alias will be normalized.

Specific ID (no forced rewriting across versions):
- `gemini-3-pro-high -> gemini-3-pro-high`
- `gemini-3-pro-low -> gemini-3-pro-low`
- `gemini-3-pro-preview -> gemini-3-pro-preview`
- `gemini-3.1-pro-high -> gemini-3.1-pro-high`
- `gemini-3.1-pro-low -> gemini-3.1-pro-low`
- `gemini-3.1-pro-preview -> gemini-3.1-pro-preview`

Generic alias (still mapped to preview entry):
- `gemini-3-pro -> gemini-3-pro-preview`
- `gemini-3.1-pro -> gemini-3.1-pro-preview`

Code location:
- `src-tauri/src/proxy/common/model_mapping.rs`

## 4) Dynamic account aware rewriting (triggered only when needed)

After selecting an account, the system will read the available models in the account's local quota to determine whether the current model is available.

The behavior is as follows:

1. Read the account JSON: `quota.models[*].name`.
2. Construct candidate fallback list only for Gemini 3/3.1 Pro family.
3. Candidate order:
   - Try the current model first
   - Then try other compatible models in the same family in the preset order
4. Select the first model that exists in the account's available collection.
5. If none of them hit, the current model remains unchanged.

Key points:
- If the request model itself is available, no remapping will occur.
- Remapping will only occur if the requested model is unavailable and a compatible candidate exists.

Code location:
- `src-tauri/src/proxy/token_manager.rs`
  - `get_available_models_from_json`
  - `build_dynamic_model_candidates`
  - `resolve_dynamic_model_for_account`

## 5) Log observation point

You can judge whether each step is triggered through the log:

- Static mapping log:
  - `[Router] system default mapping: <original> -> <mapped>`
- Dynamically rewrite log:
  - `[Dynamic-Model-Rewrite] account=<id> <from> -> <to>`

If `Dynamic-Model-Rewrite` does not appear in a certain request, it means that the account directly uses the current model.

## 6) Example

Example A (no rewriting):
- Request: `gemini-3-pro-high`
- Available models for the account include: `gemini-3-pro-high`
- Final upstream model: `gemini-3-pro-high`

Example B (rollback overwrite occurs):
- Request: `gemini-3-pro-high`
- Account unavailable: `gemini-3-pro-high`
-Account available: `gemini-3.1-pro-high`
- Final upstream model: `gemini-3.1-pro-high`

Example C (generic alias):
- Request: `gemini-3-pro`
- The static stage is first mapped to: `gemini-3-pro-preview`
- In the dynamic stage, it is decided whether to continue the rollback based on the available model of the account.

## 7) Design goals

This design meets three points at the same time:
- Specific models prioritize maintaining the user’s original intent.
- Panaliases retain historical compatibility.
- When the capabilities of multiple accounts are inconsistent, availability is improved through dynamic fallback.
