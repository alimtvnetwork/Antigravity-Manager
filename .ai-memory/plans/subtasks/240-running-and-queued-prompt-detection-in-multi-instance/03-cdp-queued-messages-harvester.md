# Subtask 03: Live DevTools CDP Composer Queued Messages Harvester in cdp_onboarding.rs

## Slug
`running-and-queued-prompt-detection-in-multi-instance - Task 03`

## Goal
Implement a DevTools CDP harvester to read in-memory composer queued messages from the live Antigravity IDE.

## Changes
- In `src-tauri/src/modules/instance/cdp_onboarding.rs`:
  - Implement `extract_queued_composer_messages_for_instance(instance_id: &str) -> Vec<String>`.
  - Connect to the DevTools WebSocket without Origin header restrictions.
  - Query DOM for `.whitespace-pre-wrap` text blocks within the `Queued Messages` composer container.
  - Return the list of queued prompt strings.
