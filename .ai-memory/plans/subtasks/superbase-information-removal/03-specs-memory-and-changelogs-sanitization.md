---
plan: superbase-information-removal
subtask: "03"
title: Specs, Memory & Changelogs Sanitization
domain: documentation-and-memory
depends_on:
  - 01-repo-secrets-vault-and-oneliner.md
citations:
  app_spec: 02-spec/21-app/superbase-information-removal/02-credential-scrub-spec.md
  architecture_spec: 02-spec/21-app/superbase-information-removal/01-architecture-spec.md
  coding_guidelines: 02-spec/02-coding-guidelines/02-canonical-size-tier.md
  strictly_avoid: 02-spec/02-coding-guidelines/strictly-avoid.md
target_files:
  - 02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-architecture-spec.md
  - 02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md
  - .ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md
  - .ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/01-supabase-dual-cluster-and-stacktrace.md
  - .ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/05-image-sharing-public-url.md
  - .ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md
  - changelog.md
  - changelog_en.md
status: pending
---

# 03 — Specs, Memory & Changelogs Sanitization

## 1. Context & Objectives

Specifications, task memory archives, subtask planning documents, and release changelogs recorded exact production credentials during previous integration phases. Leaving raw credentials in markdown artifacts poses the same security exposure as hardcoding them in source code: anyone reading architecture documents, searching the repository, or inspecting repository history can extract the project URLs and publishable anon keys.

This subtask performs systematic credential eradication across all documentation, memory plans, and release notes:
1. **Architecture Specifications**: Sanitize `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-architecture-spec.md` and `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`.
2. **AI Memory & Planning Files**: Sanitize `.ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md`, `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/01-supabase-dual-cluster-and-stacktrace.md`, `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/05-image-sharing-public-url.md`, and `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`.
3. **Release Changelogs**: Sanitize `changelog.md` and `changelog_en.md` for historical release notes.
4. **Standardized Redaction Tokens**: Enforce uniform redaction placeholders across all files:
   - `sb_publishable_[REDACTED_LOVABLE_KEY]`
   - `sb_publishable_[REDACTED_PERSONAL_KEY]`
   - `https://[REDACTED_LOVABLE_PROJECT].supabase.co`
   - `https://[REDACTED_PERSONAL_PROJECT].supabase.co`
   - `[REDACTED_LOVABLE_PROJECT_ID]`
   - `[REDACTED_PERSONAL_PROJECT_ID]`

All paths are strictly relative. No raw secret strings may appear anywhere in this document or modified files.

---

## 2. Target Files & Transformation Breakdown

### 2.1 Specification: `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-architecture-spec.md`
- **Target Areas**:
  - **Mermaid Topology Diagram** (Lines 41–42, 48–49): Replace project IDs with `[REDACTED_LOVABLE_PROJECT_ID]` and `[REDACTED_PERSONAL_PROJECT_ID]`. Replace PostgREST URLs with `https://[REDACTED_LOVABLE_PROJECT].supabase.co/rest/v1`.
  - **Endpoint Specification Table** (Lines 71–74): Replace project identifiers, raw host URLs, normalized base URLs, and publishable anon keys with standard redaction tokens.
  - **JSON Configuration Payloads** (Lines 91–92, 103–104): Replace URLs and `api_key` values with `https://[REDACTED_LOVABLE_PROJECT].supabase.co` and `sb_publishable_[REDACTED_LOVABLE_KEY]`.
  - **Normalization Test Cases** (Lines 157–159): Replace table rows containing raw URLs with redacted patterns.
  - **Storage Bucket CDN Diagram** (Lines 265–266): Replace CDN URLs with `https://[REDACTED_LOVABLE_PROJECT].supabase.co/storage/v1/object/public/...`.

### 2.2 Specification: `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`
- **Target Areas**:
  - **Context & Objectives** (Lines 13, 15, 19, 21): Replace embedded URLs and publishable anon keys with redaction placeholders.
  - **JSON Endpoint Fixtures** (Lines 88–89, 100–101): Replace `url` and `api_key` values with redacted placeholders.

### 2.3 Master Plan: `.ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md`
- **Target Areas**:
  - **User Request Section** (Lines 17–18, 21–22): Replace raw Supabase URLs and keys with redactions.
  - **Subtask Breakdown Section** (Line 59): Replace raw URLs with generic descriptions: `Primary Lovable endpoint` and `Secondary Personal endpoint`.

### 2.4 Subtask Plan: `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/01-supabase-dual-cluster-and-stacktrace.md`
- **Target Areas**:
  - **Objectives Block** (Line 23): Replace raw URLs and keys with redacted equivalents.
  - **Technical Scope** (Lines 40–41, 48–49): Redact Primary and Secondary Raw URLs and API Keys.
  - **Test Matrix Table** (Lines 156–157): In TEST-03 and TEST-04, replace project IDs with `Lovable cluster` and `Personal cluster`.

### 2.5 Subtask Plan: `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/05-image-sharing-public-url.md`
- **Target Areas**:
  - **Mermaid Router Diagram** (Lines 49–50): Replace `Host: [REDACTED_LOVABLE_PROJECT].supabase.co` and `Host: [REDACTED_PERSONAL_PROJECT].supabase.co` with `Host: [REDACTED_LOVABLE_HOST]` and `Host: [REDACTED_PERSONAL_HOST]`.
  - **Routing Algorithm Description** (Lines 87–88): Replace raw project IDs with `Primary Cluster` and `Secondary Cluster`.

### 2.6 Completed Plan: `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`
- **Target Areas**:
  - **Recorded Execution Log** (Lines 13–14): Replace raw URLs and keys with `[REDACTED_LOVABLE_URL]`, `[REDACTED_LOVABLE_KEY]`, `[REDACTED_PERSONAL_URL]`, and `[REDACTED_PERSONAL_KEY]`.

### 2.7 Changelogs: `changelog.md` and `changelog_en.md`
- **Target Areas**:
  - **English Changelog** (`changelog_en.md`, Line 785): In entry for v4.96.0, remove `https://[REDACTED_LOVABLE_PROJECT].supabase.co/rest/v1/` and `https://[REDACTED_PERSONAL_PROJECT].supabase.co/rest/v1/`. Replace with `Root Supabase (Lovable)` and `Secondary Supabase`.
  - **Chinese Changelog** (`changelog.md`, Line 930): In entry for v4.96.0, replace raw URLs with equivalent generic endpoint labels.

---

## 3. Implementation Workflow & Execution Order

1. **Phase 1: Specifications Sanitization**:
   - Scrub `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-architecture-spec.md`.
   - Scrub `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`.
2. **Phase 2: Active Plans & Subtasks Sanitization**:
   - Scrub `.ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md`.
   - Scrub `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/01-supabase-dual-cluster-and-stacktrace.md`.
   - Scrub `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/05-image-sharing-public-url.md`.
3. **Phase 3: Completed Memory & Changelog Sanitization**:
   - Scrub `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`.
   - Scrub `changelog.md` and `changelog_en.md`.
4. **Phase 4: Multi-Directory GitMap Scan**:
   - Run `gitmap aum search` across `02-spec/`, `.ai-memory/`, and root changelogs to verify 0 hits.

---

## 4. Verification Test Matrix

| Test ID | Command / Query | Scope | Expected Result |
| :--- | :--- | :--- | :--- |
| **TEST-03-01** | `gitmap aum search [REDACTED_LOVABLE_PROJECT_ID] 02-spec/` | `02-spec/` | 0 hits |
| **TEST-03-02** | `gitmap aum search [REDACTED_PERSONAL_PROJECT_ID] 02-spec/` | `02-spec/` | 0 hits |
| **TEST-03-03** | `gitmap aum search [REDACTED_LOVABLE_PROJECT_ID] .ai-memory/` | `.ai-memory/` | 0 hits (excluding subtask 04 git log instruction if needed) |
| **TEST-03-04** | `gitmap aum search [REDACTED_PERSONAL_PROJECT_ID] .ai-memory/` | `.ai-memory/` | 0 hits |
| **TEST-03-05** | `gitmap aum search [REDACTED_LOVABLE_KEY] changelog*.md` | Root changelogs | 0 hits |
| **TEST-03-06** | `gitmap aum search [REDACTED_PERSONAL_KEY] changelog*.md` | Root changelogs | 0 hits |
