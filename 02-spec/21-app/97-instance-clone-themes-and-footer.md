# 97 — Instance clone copies IDE settings, themes, and a compact footer

A duplicate instance must contain the source IDE's settings and repo databases, not only a new empty window.

## Clone

`copy_instance` copies three things from the source:

1. `User/settings.json`, `User/globalStorage`, and `User/workspaceStorage` (workspace `state.vscdb` files).
2. `.gemini/antigravity`, `.gemini/antigravity-ide`, and `.gemini/antigravity-cli`. The default instance reads these from the real user profile. A named instance reads them from `instances/<id>/home`.
3. Rows in `repo_prompts.db` for that instance id. New ids are `{old}__{target}` so the source rows stay.

Locks and cache directories are still skipped. The bound account token is still injected after the copy so the clone can sign in. That injection does not replace `settings.json` or the workspace databases.

Proof: `clone_copies_settings_workspace_db_and_gemini_repo` and `clone_repo_rows_keeps_source_and_copies_onto_target`.

## Themes

Settings has a Themes tab next to Email. Choices are the catalogue in `02-spec/07-design-system/16-theme-catalogue-and-palettes.md`: Midnight Cyber, VS Code Dark+, Tokyo Night, One Dark, GitHub Dark High Contrast, Monokai Pro, and Warm Editorial, plus Light, Dark, and System. The id is stored in `config.theme`.

## Footer and About

The accounts footer does not print "Showing X to Y of Z". Per page sits on the left. The update notice and page buttons sit in the center. The default page size is 150. About is a single compact row: small logo, version, and text links. The update channel stays on that page.
