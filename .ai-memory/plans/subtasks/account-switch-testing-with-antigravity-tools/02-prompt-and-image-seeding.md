# Subtask 02: Prompt and Image Seeding

## Objective
Register and seed an active running prompt containing deep analytical instructions and a multimodal image payload into the test instance environment. Verify pre-switch persistence in SQLite and workspace files.

## Execution Steps
1. Create a multimodal test image payload (`assets/test_diagram.png` or Base64 data URI).
2. Seed the active prompt in `repo_prompts.db:active_prompts` for the test instance:
   - Text: `"Deep analysis: inspect neural layer latency graph and layout fidelity"`
   - `image_payload`: Populated with valid Base64 image data URI.
3. Trigger pre-switch snapshot:
   - Run `cargo run --bin agm -- backup-prompts`.
4. Verify persistence:
   - Verify `has_images = 1` and `image_payload` populated in `backup-prompts.db:prompt_backups`.
   - Verify creation of `.antigravity_resume_task.<test_instance>.json` and `.antigravity_resume_task.json`.
