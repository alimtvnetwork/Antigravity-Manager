/**
 * Verification & Rerun Prompt Generation Engine
 *
 * Implements the 5-Chapter prompt architecture for task completion audits,
 * release engineering verification, architectural confidence assessments,
 * and iteration-tracked rerun nudges.
 */

import { useErrorStore } from '../../../stores/error-store';

/**
 * Supported verification template types / preset keys.
 */
export type VerificationTemplateType =
    | 'is-it-done'
    | 'is-it-released'
    | 'tested-e2e'
    | 'tested-e2e-and-released'
    | 'are-you-confident'
    | 'edge-cases-and-tests'
    | 'security-audit'
    | 'pr-review-ready'
    | 'test-and-coverage'
    | 'refactor-dry'
    | 'run-100-steps'
    | 'edge-cases'
    | 'verify-tests'
    | 'rerun-nudge'
    | 'none';

/**
 * Unified Suffix Template model (matches SQLite table and localStorage schema).
 */
export interface SuffixTemplate {
    id: string;
    name: string;
    description: string;
    category:
        | 'verification'
        | 'release'
        | 'quality'
        | 'robustness'
        | 'security'
        | 'governance'
        | 'testing'
        | 'architecture'
        | 'custom';
    template: string;
    is_default?: boolean;
    created_at?: string;
    updated_at?: string;
}

/**
 * Metadata descriptor for verification options rendered in UI dropdowns.
 */
export interface VerificationOptionMeta {
    value: VerificationTemplateType;
    label: string;
    dropdownLabel?: string;
    badgeLabel?: string;
    description?: string;
    category?: SuffixTemplate['category'];
    iconName?:
        | 'check-circle'
        | 'rocket'
        | 'shield-check'
        | 'alert-triangle'
        | 'lock'
        | 'git-pull-request'
        | 'terminal'
        | 'sparkles'
        | 'repeat'
        | 'slash';
}

/**
 * Context supplied when generating a rich verification prompt.
 */
export interface VerificationTemplateContext {
    /** The complete original prompt text to embed into Chapter 4. */
    originalPrompt: string;
    /** Template category / kind discriminator. */
    kind?: string;
    /** Template type discriminator alias. */
    type?: string;
    /** Target instance identifier (e.g. 'default' or instance UUID). */
    instanceId?: string;
    /** Repository path for relative path anchoring. */
    repoPath?: string;
    /** Target conversation ID. */
    convId?: string;
    /** Conversation display title. */
    convTitle?: string;
    /** Current Git branch if known. */
    branch?: string;
    /** Touched files or code symbols modified during execution. */
    touchedFiles?: string[];
    /** Current round for rerun nudges (1-indexed). */
    currentRound?: number;
    /** Total scheduled rounds for rerun nudges. */
    totalRounds?: number;
}

/**
 * Context supplied specifically for rerun nudges.
 */
export interface RerunTemplateContext {
    originalPrompt?: string;
    prompt?: string;
    round?: number;
    total?: number;
    instanceId?: string;
    repoPath?: string;
    convId?: string;
    customNudgeMessage?: string;
}

/**
 * 8 Expanded Domain-Specific Verification Options for UI selection in DetailHeader.
 */
export const VERIFICATION_OPTIONS: ReadonlyArray<VerificationOptionMeta> = [
    {
        value: 'none',
        label: 'None (Send as is)',
        dropdownLabel: 'None',
        badgeLabel: 'Default',
        description: 'Send prompt without verification envelope',
        category: 'custom',
        iconName: 'slash',
    },
    {
        value: 'is-it-done',
        label: 'Is it done? (Audit Checklist)',
        dropdownLabel: 'Done?',
        badgeLabel: 'Done',
        description: 'Comprehensive completion audit & evidence scorecard',
        category: 'verification',
        iconName: 'check-circle',
    },
    {
        value: 'is-it-released',
        label: 'Is it released? (Release Gates)',
        dropdownLabel: 'Released?',
        badgeLabel: 'Release',
        description: 'Release channel, 14-location SemVer, attribution & gate checks',
        category: 'release',
        iconName: 'rocket',
    },
    {
        value: 'tested-e2e',
        label: 'Have you tested these below instructions with end-to-end testing?',
        dropdownLabel: 'E2E Tested?',
        badgeLabel: 'E2E Test',
        description: 'Strict pre-deployment verification: Have you tested these below instructions with end-to-end testing?',
        category: 'testing',
        iconName: 'terminal',
    },
    {
        value: 'tested-e2e-and-released',
        label: 'Have you tested the below instructions with end-to-end testing and released it?',
        dropdownLabel: 'E2E & Released?',
        badgeLabel: 'E2E & Release',
        description: 'Strict pre-deployment verification: Have you tested the below instructions with end-to-end testing and released it?',
        category: 'release',
        iconName: 'rocket',
    },
    {
        value: 'are-you-confident',
        label: 'Are you confident? (Quality Audit)',
        dropdownLabel: 'Sure?',
        badgeLabel: 'Quality',
        description: 'Code quality, <= 15-line functions, positive booleans & error routing',
        category: 'quality',
        iconName: 'shield-check',
    },
    {
        value: 'edge-cases-and-tests',
        label: 'Edge Cases & Tests',
        dropdownLabel: 'Edge cases',
        badgeLabel: 'Tests',
        description: 'Boundary checks, concurrency, recovery ladders & test suite gates',
        category: 'robustness',
        iconName: 'alert-triangle',
    },
    {
        value: 'security-audit',
        label: 'Security Audit & Zero-Trust',
        dropdownLabel: 'Security',
        badgeLabel: 'Security',
        description: 'Secrets scrub, credential protection, input validation & proxy guard',
        category: 'security',
        iconName: 'lock',
    },
    {
        value: 'pr-review-ready',
        label: 'PR Review Readiness',
        dropdownLabel: 'PR Review',
        badgeLabel: 'Review',
        description: 'Single problem scope, clean revertable commits & PR template match',
        category: 'governance',
        iconName: 'git-pull-request',
    },
    {
        value: 'test-and-coverage',
        label: 'Tests & Coverage Verification',
        dropdownLabel: 'Tests & Cov',
        badgeLabel: 'Coverage',
        description: 'Targeted unit tests, mock isolation, error branch coverage & no OS mutation',
        category: 'testing',
        iconName: 'terminal',
    },
    {
        value: 'refactor-dry',
        label: 'Refactor DRY & Code Cleanliness',
        dropdownLabel: 'Refactor DRY',
        badgeLabel: 'DRY',
        description: 'Pattern deduplication, clean helper extraction & zero logic drift',
        category: 'architecture',
        iconName: 'sparkles',
    },
    {
        value: 'run-100-steps',
        label: '100 Steps (V6 execute parent task)',
        dropdownLabel: '100 Steps',
        badgeLabel: '100 Steps',
        description: 'Wrap this prompt in execute-parent-task-with-n-steps-v6 and run it as 100+ sequential steps',
        category: 'architecture',
        iconName: 'sparkles',
    },
];

/**
 * Default option when none is explicitly selected.
 */
export const DEFAULT_VERIFICATION_TYPE: VerificationTemplateType = 'none';

/**
 * Storage keys for custom templates and default preset.
 */
export const CUSTOM_TEMPLATES_STORAGE_KEY = 'agm_prompt_custom_suffix_templates';
export const DEFAULT_TEMPLATE_STORAGE_KEY = 'agm_prompt_default_suffix_id';

/**
 * Maps raw or legacy suffix strings to a structured VerificationTemplateType.
 */
export function parseVerificationTypeFromSuffix(rawSuffix: string): VerificationTemplateType {
    if (!rawSuffix || typeof rawSuffix !== 'string') {
        return 'none';
    }

    const normalized = rawSuffix.trim().toLowerCase();

    if (normalized === 'none' || normalized === 'none (send as is)' || normalized === '') {
        return 'none';
    }

    if (
        normalized === 'is-it-done' ||
        normalized === 'is it done?' ||
        normalized === 'done?' ||
        normalized === 'done' ||
        normalized.includes('is-it-done')
    ) {
        return 'is-it-done';
    }

    if (
        normalized === 'is-it-released' ||
        normalized === 'is it released?' ||
        normalized === 'released?' ||
        normalized === 'release' ||
        normalized.includes('is-it-released')
    ) {
        return 'is-it-released';
    }

    if (
        normalized === 'tested-e2e' ||
        normalized === 'have you tested these below instructions with end-to-end testing?' ||
        normalized.includes('tested-e2e') ||
        normalized.includes('with end-to-end testing?')
    ) {
        return 'tested-e2e';
    }

    if (
        normalized === 'tested-e2e-and-released' ||
        normalized === 'have you tested the below instructions with end-to-end testing and released it?' ||
        normalized.includes('tested-e2e-and-released') ||
        normalized.includes('and released it?')
    ) {
        return 'tested-e2e-and-released';
    }

    if (
        normalized === 'are-you-confident' ||
        normalized === 'are you sure about it?' ||
        normalized === 'sure?' ||
        normalized === 'confident' ||
        normalized.includes('are-you-confident')
    ) {
        return 'are-you-confident';
    }

    if (
        normalized === 'edge-cases-and-tests' ||
        normalized === 'edge-cases' ||
        normalized === 'edge cases' ||
        normalized === 'double check all edge cases'
    ) {
        return 'edge-cases-and-tests';
    }

    if (
        normalized === 'security-audit' ||
        normalized === 'security' ||
        normalized.includes('security')
    ) {
        return 'security-audit';
    }

    if (
        normalized === 'pr-review-ready' ||
        normalized === 'pr review' ||
        normalized.includes('pr-review')
    ) {
        return 'pr-review-ready';
    }

    if (
        normalized === 'test-and-coverage' ||
        normalized === 'verify-tests' ||
        normalized === 'tests & cov' ||
        normalized.includes('coverage')
    ) {
        return 'test-and-coverage';
    }

    if (
        normalized === 'refactor-dry' ||
        normalized === 'dry' ||
        normalized.includes('refactor-dry')
    ) {
        return 'refactor-dry';
    }

    if (
        normalized === 'run-100-steps' ||
        normalized === '100 steps' ||
        normalized === '100-step execute' ||
        normalized.includes('n-steps') ||
        normalized.includes('100 steps')
    ) {
        return 'run-100-steps';
    }

    return 'none';
}

/**
 * Retrieves the verification option metadata by type.
 */
export function getVerificationOption(type: string): VerificationOptionMeta | undefined {
    const parsed = parseVerificationTypeFromSuffix(type);
    return VERIFICATION_OPTIONS.find((opt) => opt.value === parsed);
}

/**
 * Built-in 8 presets represented as SuffixTemplate objects.
 */
export const BUILTIN_SUFFIX_TEMPLATES: ReadonlyArray<SuffixTemplate> = [
    {
        id: 'is-it-done',
        name: 'Done? (Is it done?)',
        description: 'Comprehensive completion audit & evidence scorecard',
        category: 'verification',
        template: [
            '### Task Completion Audit:',
            '- Verify all requirement acceptance criteria against grounded file inspection.',
            '- Provide line-numbered references (e.g. `src/...#L10-L25`) for all modifications.',
            '- Confirm zero orphan TODOs and clean working tree before marking done.',
        ].join('\n'),
    },
    {
        id: 'is-it-released',
        name: 'Released? (Release Gates)',
        description: '14-location SemVer sync, strict @aukgit attribution & CI pre-flight',
        category: 'release',
        template: [
            '### Release Engineering Checklist:',
            '- Check branch alignment (main for stable, beta for preview).',
            '- Synchronize 14 version manifest locations via `npm run bump`.',
            '- Attributions strictly `@aukgit` (`(Thanks to @aukgit)`).',
            '- Synchronize README and README_EN update logs.',
            '- Run fmt, clippy, and build pre-flight gates.',
        ].join('\n'),
    },
    {
        id: 'tested-e2e',
        name: 'E2E Tested? (End-to-End Verification)',
        description: 'Strict pre-deployment: Have you tested these below instructions with end-to-end testing?',
        category: 'testing',
        template: [
            '### End-to-End (E2E) Testing Verification:',
            '- Have you tested these below instructions with end-to-end testing?',
            '- Run end-to-end verification flows across polyglot components and verify all pass green.',
            '- Inspect command logs and verify zero regressions in runtime behavior.',
        ].join('\n'),
    },
    {
        id: 'tested-e2e-and-released',
        name: 'E2E & Released? (Deployment Clearance)',
        description: 'Strict pre-deployment: Have you tested the below instructions with end-to-end testing and released it?',
        category: 'release',
        template: [
            '### End-to-End (E2E) & Release Clearance:',
            '- Have you tested the below instructions with end-to-end testing and released it?',
            '- Verify full 14-location SemVer sync and release branch invariants.',
            '- Confirm pre-flight gates (cargo fmt, clippy, build) are 100% green before release.',
        ].join('\n'),
    },
    {
        id: 'are-you-confident',
        name: 'Sure? (Quality Audit)',
        description: 'Code quality, <= 15-line functions, positive booleans & error routing',
        category: 'quality',
        template: [
            '### Code Quality & Confidence Audit:',
            '- Keep core functions bounded to <= 15 lines with single responsibility.',
            '- Flatten nested conditionals with early returns and guard clauses.',
            '- Enforce positive boolean polarity (no double negatives).',
            '- Route all errors through AppError and useErrorStore (no bare catch {}).',
        ].join('\n'),
    },
    {
        id: 'edge-cases-and-tests',
        name: 'Edge cases (Boundary Checks)',
        description: 'Boundary checks, job breakaway fallbacks, timeouts & mock isolation',
        category: 'robustness',
        template: [
            '### Edge Cases & Defensive Architecture:',
            '- Validate empty inputs, collection bounds, and null/undefined handling.',
            '- Ensure Job Object breakaway fallbacks (spawn_with_job_breakaway_fallback).',
            '- Bound all network and process waits with explicit timeouts.',
            '- Verify multi-instance profile sandboxing and conscious PID matching.',
        ].join('\n'),
    },
    {
        id: 'security-audit',
        name: 'Security (Secrets & Zero-Trust)',
        description: 'Secrets scrub, credential protection, input validation & zero-trust',
        category: 'security',
        template: [
            '### Security & Zero-Trust Audit:',
            '- Scan for leaked secrets, API keys, tokens, or hardcoded passwords.',
            '- Ensure reverse proxy IP security and CIDR bounds.',
            '- Sanitize untrusted user inputs before SQLite queries or shell execution.',
            '- Validate AES-256-GCM encrypted credential vault isolation.',
        ].join('\n'),
    },
    {
        id: 'pr-review-ready',
        name: 'PR Review (Single Scope & Clean Commits)',
        description: 'Single problem scope, clean revertable commits & PR template match',
        category: 'governance',
        template: [
            '### PR Readiness & Governance Gate:',
            '- PR must address a single cohesive problem class.',
            '- Each commit must be self-contained and individually revertable.',
            '- Clean local trial-and-error noise into final-state commits.',
            '- Complete .github/pull_request_template.md sections.',
        ].join('\n'),
    },
    {
        id: 'test-and-coverage',
        name: 'Tests & Cov (Mock Isolation & Suites)',
        description: 'Targeted unit tests, mock isolation, error branch coverage & no OS mutation',
        category: 'testing',
        template: [
            '### Unit Testing & Mock Isolation:',
            '- Run targeted unit tests for touched modules locally.',
            '- Never execute real OS shutdown, reboot, or destructive system calls.',
            '- Inject mock executors and verify mock call durations.',
            '- Verify both happy-path and error-branch coverage.',
        ].join('\n'),
    },
    {
        id: 'refactor-dry',
        name: 'Refactor DRY (Deduplication & Clean Helpers)',
        description: 'Pattern deduplication, clean helper extraction & zero logic drift',
        category: 'architecture',
        template: [
            '### Code DRYness & Architectural Hygiene:',
            '- Identify repeated logic blocks across modules and extract shared helpers.',
            '- Eliminate redundant slice wrappers using variadic / spread patterns.',
            '- Preserve original business logic, comments, and docstrings.',
            '- Verify zero functional regression after extraction.',
        ].join('\n'),
    },
    {
        id: 'run-100-steps',
        name: '100 Steps (V6 parent task)',
        description: 'Break the prompt into 100+ sequential steps via execute-parent-task-with-n-steps-v6',
        category: 'architecture',
        template: [
            'Must follow and spawn agent using',
            '[execute-parent-task-with-n-steps-v6](file;.agents/skills/execute-parent-task-with-n-steps-v6)',
            '',
            'Break the user request into 100+ sequential steps, execute them continuously, and do not stop until the task is done, tested, and released.',
        ].join('\n'),
    },
];

/**
 * Retrieve custom suffix templates from localStorage with fallback.
 */
export function getCustomSuffixTemplates(): SuffixTemplate[] {
    try {
        if (typeof window === 'undefined' || !window.localStorage) {
            return [];
        }
        const raw = window.localStorage.getItem(CUSTOM_TEMPLATES_STORAGE_KEY);
        if (!raw) return [];
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) {
            return parsed as SuffixTemplate[];
        }
        return [];
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.getCustomSuffixTemplates',
            triggerAction: 'load_custom_templates',
        });
        return [];
    }
}

/**
 * Save or update a custom suffix template in localStorage.
 */
export function saveCustomSuffixTemplate(template: SuffixTemplate): void {
    try {
        if (typeof window === 'undefined' || !window.localStorage) return;
        const current = getCustomSuffixTemplates();
        const now = new Date().toISOString();
        const existingIdx = current.findIndex((t) => t.id === template.id);
        const updatedTemplate: SuffixTemplate = {
            ...template,
            updated_at: now,
            created_at: template.created_at || now,
        };

        let nextList: SuffixTemplate[];
        if (existingIdx >= 0) {
            nextList = [...current];
            nextList[existingIdx] = updatedTemplate;
        } else {
            nextList = [...current, updatedTemplate];
        }

        window.localStorage.setItem(CUSTOM_TEMPLATES_STORAGE_KEY, JSON.stringify(nextList));
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.saveCustomSuffixTemplate',
            triggerAction: 'save_custom_template',
        });
    }
}

/**
 * Delete a custom suffix template from localStorage.
 */
export function deleteCustomSuffixTemplate(id: string): void {
    try {
        if (typeof window === 'undefined' || !window.localStorage) return;
        const current = getCustomSuffixTemplates();
        const nextList = current.filter((t) => t.id !== id);
        window.localStorage.setItem(CUSTOM_TEMPLATES_STORAGE_KEY, JSON.stringify(nextList));
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.deleteCustomSuffixTemplate',
            triggerAction: 'delete_custom_template',
        });
    }
}

/**
 * Retrieve all suffix templates (both built-in presets and user custom templates).
 */
export function getAllSuffixTemplates(): SuffixTemplate[] {
    const custom = getCustomSuffixTemplates();
    return [...BUILTIN_SUFFIX_TEMPLATES, ...custom];
}

/**
 * Get the currently configured default suffix template ID.
 */
export function getDefaultSuffixTemplateId(): string {
    try {
        if (typeof window === 'undefined' || !window.localStorage) {
            return DEFAULT_VERIFICATION_TYPE;
        }
        return window.localStorage.getItem(DEFAULT_TEMPLATE_STORAGE_KEY) || DEFAULT_VERIFICATION_TYPE;
    } catch {
        return DEFAULT_VERIFICATION_TYPE;
    }
}

/**
 * Set the default suffix template ID.
 */
export function setDefaultSuffixTemplateId(id: string): void {
    try {
        if (typeof window === 'undefined' || !window.localStorage) return;
        window.localStorage.setItem(DEFAULT_TEMPLATE_STORAGE_KEY, id);
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.setDefaultSuffixTemplateId',
            triggerAction: 'set_default_template',
        });
    }
}

/**
 * Generates Chapter 1: 🚨 High Priority Audit Mandate
 */
function buildChapter1AuditMandate(): string {
    return [
        '# 🚨 CHAPTER 1: HIGH PRIORITY AUDIT MANDATE',
        '',
        '> [!IMPORTANT]',
        '> **Audit Authority & Non-Negotiable Precedence:**',
        '> You are acting as the Lead Verification Auditor. Your objective is to conduct a strict,',
        '> evidence-based verification of the task described below.',
        '>',
        '> - **Anti-Hallucination Policy:** Do NOT assume, extrapolate, or claim completion without inspecting the exact files and running the necessary checks.',
        '> - **Zero Silent Failures:** Every check must yield explicit verifiable proof. If any step is unfinished, broken, or untested, explicitly report it as FAILED with actionable diagnostics.',
    ].join('\n');
}

/**
 * Generates Chapter 2 checklist tailored to the requested verification type.
 */
function buildChapter2Checklist(type: VerificationTemplateType, customTemplateText?: string): string {
    if (customTemplateText) {
        return [
            '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
            '',
            '### Custom Verification Directives:',
            customTemplateText,
        ].join('\n');
    }

    switch (type) {
        case 'is-it-done':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.1: \'Is It Done?\' Verification Matrix',
                '- [ ] **1. Exact Code Inspection:** Have you read and cited all modified source files with line-numbered links (e.g. `src/...#L10-L25`)?',
                '- [ ] **2. Root Cause Articulation:** Have you clearly stated the 1-sentence root cause of the issue?',
                '- [ ] **3. Surgical Implementation:** Are all modifications strictly scoped to the problem without unnecessary code churn or speculative edits?',
                '- [ ] **4. Concrete Verification:** Have you run targeted tests or verified UI/CLI behavior with real command output (zero mock guesses)?',
                '- [ ] **5. Memory & Plan Sync:** Are all relevant plans under `.ai-memory/plans/` updated, all completed action items marked checked (`- [x]`), and no orphan TODOs remaining?',
            ].join('\n');

        case 'is-it-released':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.2: \'Is It Released?\' Governance & Release Gate Matrix',
                '- [ ] **1. Release Branch Separation:** Is the current branch strictly `main` (for official stable releases) or `beta` (for preview releases)?',
                '- [ ] **2. 14-Location SemVer Sync:** Are all 14 project manifest version fields atomically updated via `npm run bump`?',
                '- [ ] **3. Strict Attribution Invariant:** Are changelogs attributed strictly to `@aukgit` (`(Thanks to @aukgit)`) with ZERO other GitHub user handles (`@...`)?',
                '- [ ] **4. README Synchronization:** Are both `README.md` (under "## 📝 Update log") and `README_EN.md` (under "## 📝 Changelog") update logs synchronized with the release notes?',
                '- [ ] **5. Pre-flight Quality Gates:** Have `cd src-tauri && cargo fmt -- --check`, `cd src-tauri && cargo clippy --all-targets --all-features`, and `npm run build` all passed cleanly?',
                '- [ ] **6. Git Tag Precision:** Does the git tag strictly match `CHANGELOG.md` heading character-for-character (`vX.Y.Z` on main, `vX.Y.Z-beta.N` on beta)?',
                '- [ ] **7. Distribution Scripts:** Are `install.ps1` and `install.sh` verified for the target release?',
                '- [ ] **8. Thinking Store Invalidation Toggle:** Is `SUGGESTION_DELETE_THINKING_STORE` correctly configured in `SuggestionDeleteThinkingModal.tsx`?',
            ].join('\n');

        case 'tested-e2e':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.9: End-to-End (E2E) Pre-Deployment Verification',
                '> **Mandatory Question:** Have you tested these below instructions with end-to-end testing?',
                '- [ ] **1. End-to-End Test Execution:** Have you verified the full workflow from start to finish with concrete end-to-end tests?',
                '- [ ] **2. Polyglot Pipeline Status:** Are frontend (`npm run build`), Rust backend (`cargo clippy`), and CLI commands passing without errors?',
                '- [ ] **3. Runtime Verification:** Have you verified live state transitions and error logging in the running application?',
                '- [ ] **4. Zero Regressions:** Are all existing behaviors preserved without unintended regressions?',
            ].join('\n');

        case 'tested-e2e-and-released':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.10: End-to-End (E2E) Testing & Release Clearance',
                '> **Mandatory Question:** Have you tested the below instructions with end-to-end testing and released it?',
                '- [ ] **1. End-to-End Test Execution:** Have you verified the full workflow from start to finish with concrete end-to-end tests?',
                '- [ ] **2. Release Channel & Branch:** Is the release tag on the proper branch (`main` for stable, `beta` for preview)?',
                '- [ ] **3. Atomic SemVer Sync:** Are all 14 version manifest locations bumped and synchronized?',
                '- [ ] **4. Changelog & Attribution Integrity:** Is attribution strictly `@aukgit` and are README release notes updated?',
                '- [ ] **5. Release Gate Clearance:** Have all CI/CD pre-flight checks and packaging tests passed 100% green?',
            ].join('\n');

        case 'are-you-confident':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.3: \'Are You Confident?\' Architectural & Code Quality Matrix',
                '- [ ] **1. Compact Functions:** Are functions bounded to <= 15 lines of core logic with single responsibility?',
                '- [ ] **2. Guard Clauses & Early Returns:** Are nested `if/else` structures flattened with guard returns and clean constructor returns?',
                '- [ ] **3. Positive Boolean Polarity:** Are all boolean flags positive, implicit (`if (isReady)`), and free of double negatives (`!isDisabled`)?',
                '- [ ] **4. Zero Silent Errors:** Does every catch block and error result route to the error store (`captureException`/`trackWarning`) or `AppError` (no bare `catch {}`)?',
                '- [ ] **5. Vertical Rhythm:** Is vertical line spacing enforced (blank lines around blocks and before returns)?',
                '- [ ] **6. Relative Paths:** Are all referenced file paths strictly repository-relative (`src/...`, `02-spec/...`) without absolute machine paths?',
            ].join('\n');

        case 'edge-cases':
        case 'edge-cases-and-tests':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.4: Edge Cases & Test Coverage Matrix',
                '- [ ] **1. Boundary Values:** Have empty data states, null/undefined inputs, and collection bounds been tested?',
                '- [ ] **2. Error Recovery Ladders:** Are network timeouts, offline states, and service failure ladders robustly handled?',
                '- [ ] **3. Concurrency & Sandboxing:** Are multi-instance profile sandboxing, conscious PID matching, and lockfiles protected against race conditions?',
                '- [ ] **4. Encoding & Media Preservation:** Are unicode, emojis, and embedded rich markdown images fully preserved?',
                '- [ ] **5. Test Execution Gate:** Have `npm run build`, `cargo clippy`, and targeted test suites passed with zero regressions?',
            ].join('\n');

        case 'security-audit':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.5: Security & Zero-Trust Audit Matrix',
                '- [ ] **1. Secrets & Credentials:** Zero hardcoded tokens, OAuth client secrets, or private keys in source code or committed artifacts.',
                '- [ ] **2. Network & CIDR Bounds:** Reverse proxy enforces IP allowlists/blacklists, curfew windows, and Cloudflared Zero Trust tunnels.',
                '- [ ] **3. Input Sanitization:** All user parameters, paths, and CLI arguments sanitized against directory traversal and command injection.',
                '- [ ] **4. Safe Keyring & Vault Storage:** Credentials encrypted with AES-256-GCM in separate security SQLite vaults.',
            ].join('\n');

        case 'pr-review-ready':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.6: PR Review Readiness Matrix',
                '- [ ] **1. Single Problem Scope:** The changeset addresses a single, well-defined problem class without mixing unrelated fixes.',
                '- [ ] **2. Self-Contained Commits:** Commits are cleanly grouped, individually revertable, and free from intermediate trial-and-error noise.',
                '- [ ] **3. Pull Request Template:** Checklist completed for problem classification, behavior alterations, unverified paths, and rollback strategy.',
                '- [ ] **4. Documentation Alignment:** All touched components, specs, and READMEs reflect the actual code state.',
            ].join('\n');

        case 'test-and-coverage':
        case 'verify-tests':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.7: Tests & Coverage Matrix',
                '- [ ] **1. Unit Test Execution:** All targeted unit tests for modified components pass with zero failures.',
                '- [ ] **2. Mock Isolation Invariant:** Unit tests NEVER invoke real OS shutdown, restart, or heavy unmocked hardware APIs.',
                '- [ ] **3. Error Path Coverage:** Both error pathways (try/catch, Result Err) and happy pathways are exercised.',
                '- [ ] **4. Pre-flight Verification:** Local checks (`cargo fmt`, `cargo clippy`, `npm run build`) execute cleanly.',
            ].join('\n');

        case 'refactor-dry':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.8: Refactor DRY & Code Cleanliness Matrix',
                '- [ ] **1. Pattern Deduplication:** Duplicate code patterns identified across files and consolidated into shared utility helpers.',
                '- [ ] **2. Parameter Spread & Variadics:** Artificial array wrappers eliminated in favor of spread/variadic arguments.',
                '- [ ] **3. Zero Semantic Drift:** Pure refactoring with 100% preservation of existing business logic and behavior.',
                '- [ ] **4. Documentation Integrity:** Preserved all existing comments, invariants, and docstrings.',
            ].join('\n');

        case 'run-100-steps':
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Chapter 2.10: 100-Step Parent Task Execution',
                'Must follow and spawn agent using',
                '[execute-parent-task-with-n-steps-v6](file;.agents/skills/execute-parent-task-with-n-steps-v6)',
                '',
                '- [ ] **1. Skill Load:** Load `.agents/skills/execute-parent-task-with-n-steps-v6` before executing the original prompt.',
                '- [ ] **2. Breakdown:** Split the original request into 100+ sequential, independently revertable steps.',
                '- [ ] **3. Continuous Loop:** Execute without pausing for permission except on true blockers; keep A=3 workers saturated.',
                '- [ ] **4. FIFO Restore:** Enqueue, restore, and dispatch with `ORDER BY created_at ASC, id ASC`.',
                '- [ ] **5. Done & Released:** Finish only after end-to-end proof and a relative-path changelog attributed to `@aukgit`.',
            ].join('\n');

        default:
            return [
                '# 📊 CHAPTER 2: GROUNDED VERIFICATION MATRIX & ACTIONABLE CHECKLIST',
                '',
                '### Standard Verification Checklist',
                '- [ ] **1. Source Inspection:** Verify touched files and lines against task requirements.',
                '- [ ] **2. Regression Guard:** Ensure existing features and invariants remain intact.',
                '- [ ] **3. Evidence Capture:** Record concrete execution proof for all assertions.',
            ].join('\n');
    }
}

/**
 * Generates Chapter 3: 🔍 Environment, Git & Release Telemetry Gates
 */
function buildChapter3TelemetryGates(): string {
    return [
        '# 🔍 CHAPTER 3: ENVIRONMENT, GIT & RELEASE TELEMETRY GATES',
        '',
        'Run or verify the following diagnostic commands before completing your report:',
        '1. **Working Tree Cleanliness:** Check `git status --porcelain` to identify any uncommitted, modified, or untracked files.',
        '2. **Active Branch Validation:** Check `git branch --show-current` to confirm alignment with your task scope (`main` vs `beta`).',
        '3. **Pipeline Health & Telemetry:** Ensure recent CI/CD pipeline steps are not failing (`gitmap pe -t` or local pre-flight runner).',
    ].join('\n');
}

/**
 * Generates Chapter 4: 📋 Verbatim Original Prompt Embedding
 */
function buildChapter4OriginalPrompt(originalPrompt: string, touchedFiles?: string[]): string {
    const trimmed = (originalPrompt || '').trim();
    const filesSection =
        touchedFiles && touchedFiles.length > 0
            ? [
                  '',
                  '### Referenced / Touched Context Files:',
                  ...touchedFiles.map((f) => `- \`${f}\``),
                  '',
              ].join('\n')
            : '';

    return [
        '# 📋 CHAPTER 4: VERBATIM ORIGINAL PROMPT & TASK SPECIFICATION',
        filesSection,
        'The following is the complete, un-truncated original task specification that must be audited and verified:',
        '',
        '---',
        '<<<ORIGINAL_TASK_PROMPT_START>>>',
        trimmed,
        '<<<ORIGINAL_TASK_PROMPT_END>>>',
        '---',
    ].join('\n');
}

/**
 * Generates Chapter 5: 📝 Concluding Evidence Contract
 */
function buildChapter5EvidenceContract(): string {
    return [
        '# 📝 CHAPTER 5: EVIDENCE SCORECARD CONTRACT',
        '',
        'You MUST conclude your audit with the following Evidence Scorecard table:',
        '',
        '| Verification Gate | Status | File / Line Reference | Verifiable Proof / Output |',
        '| :--- | :---: | :--- | :--- |',
        '| Codebase Citation & Reading | PASS / FAIL | `src/...#L12-L45` | File inspected and verified |',
        '| 1-Sentence Root Cause | PASS / FAIL | N/A | Diagnostic statement provided |',
        '| Surgical Code Change | PASS / FAIL | `src/...` | Zero extraneous edits or code churn |',
        '| Quality Gates (Clippy / Build) | PASS / FAIL | Terminal Output | Build passes cleanly |',
        '| Coding Guidelines (Booleans/Errors) | PASS / FAIL | `src/...` | Positive booleans, error store routed |',
        '| Memory & Plan Synchronization | PASS / FAIL | `.ai-memory/...` | Subtask marked COMPLETED |',
        '',
        '### Concluding Verdict:',
        '- **ALL_CRITERIA_MET:** `[TRUE | FALSE]`',
        '- **BLOCKING_ISSUES:** `[None | Describe any unresolved issues]`',
        '- **RECOMMENDED_NEXT_STEP:** `[Proceed to next subtask | Fix failing items | Release]`',
    ].join('\n');
}

/**
 * Builds the complete 5-Chapter verification prompt string.
 * Supports both options object and (type, context) call patterns.
 */
export function buildRichVerificationPrompt(
    optionsOrType:
        | string
        | {
              kind?: string;
              type?: string;
              originalPrompt: string;
              convTitle?: string;
              convId?: string;
              repoPath?: string;
              instanceId?: string;
              branch?: string;
              touchedFiles?: string[];
              currentRound?: number;
              totalRounds?: number;
          },
    context?: VerificationTemplateContext
): string {
    try {
        let rawKind = 'is-it-done';
        let originalPrompt = '';
        let touchedFiles: string[] | undefined;

        if (typeof optionsOrType === 'object' && optionsOrType !== null) {
            rawKind = optionsOrType.kind || optionsOrType.type || 'is-it-done';
            originalPrompt = optionsOrType.originalPrompt || '';
            touchedFiles = optionsOrType.touchedFiles;
        } else if (typeof optionsOrType === 'string') {
            rawKind = optionsOrType;
            originalPrompt = context?.originalPrompt || '';
            touchedFiles = context?.touchedFiles;
        }

        const templateType = parseVerificationTypeFromSuffix(rawKind);
        if (templateType === 'none' && !rawKind.startsWith('custom:')) {
            return originalPrompt;
        }

        // Check if rawKind refers to a custom template
        let customTemplateText: string | undefined;
        if (templateType === 'none') {
            const allTemplates = getAllSuffixTemplates();
            const found = allTemplates.find((t) => t.id === rawKind);
            if (found) {
                customTemplateText = found.template;
            }
        }

        return [
            buildChapter1AuditMandate(),
            '',
            buildChapter2Checklist(templateType, customTemplateText),
            '',
            buildChapter3TelemetryGates(),
            '',
            buildChapter4OriginalPrompt(originalPrompt, touchedFiles),
            '',
            buildChapter5EvidenceContract(),
        ].join('\n');
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.buildRichVerificationPrompt',
            triggerAction: 'buildRichVerificationPrompt',
        });
        const fallbackPrompt =
            typeof optionsOrType === 'object' && optionsOrType !== null
                ? optionsOrType.originalPrompt || ''
                : context?.originalPrompt || '';
        return fallbackPrompt;
    }
}

/**
 * Canonical alias for generating a 5-Chapter AI task verification prompt.
 */
export function generateVerificationPrompt(context: VerificationTemplateContext): string {
    return buildRichVerificationPrompt(context);
}

/**
 * Builds an iteration-tracked rerun nudge prompt.
 * Supports both (originalPrompt, round, total) and (context) call signatures.
 */
export function buildRerunPrompt(
    promptOrContext: string | RerunTemplateContext,
    round?: number,
    total?: number
): string {
    try {
        let originalPrompt = '';
        let currentRound = 1;
        let totalRounds = 1;
        let customNudge = '';

        const isNudgeText = (text: string): boolean => {
            const clean = text.toLowerCase().trim();
            if (clean.includes('# 🔄 rerun execution nudge')) {
                return false;
            }
            return (
                clean === 'is it done?' ||
                clean === 'is it published?' ||
                clean === 'is it finished?' ||
                clean === 'is it released?' ||
                clean === 'is it completed?' ||
                clean === 'done?' ||
                clean === 'published?' ||
                clean === 'finished?' ||
                clean === 'released?' ||
                clean === 'completed?' ||
                clean.startsWith('is it done') ||
                clean.startsWith('is it published') ||
                clean.startsWith('is it finished') ||
                clean.startsWith('is it released') ||
                clean.startsWith('is it completed') ||
                clean.startsWith('quick status check') ||
                clean.startsWith('has this been') ||
                clean.startsWith('can you check') ||
                clean.startsWith('please verify') ||
                clean.startsWith('did all tests') ||
                clean.startsWith('are all requirements') ||
                clean.startsWith('please confirm') ||
                clean.startsWith('please double check') ||
                clean.startsWith('is there any pending') ||
                clean.startsWith('run comprehensive verification') ||
                clean.startsWith('check pending tasks') ||
                clean.startsWith('verify build cleanliness') ||
                clean.startsWith('confirm all changes') ||
                clean.startsWith('did you finish') ||
                clean.startsWith('is the build clean') ||
                clean.startsWith('please review and finalize') ||
                clean.startsWith('checking in') ||
                clean.startsWith('status update') ||
                clean.includes('status check') ||
                clean.includes('progress update') ||
                clean.includes('ping me when') ||
                clean.includes('have you tested these below instructions with end-to-end testing') ||
                clean.includes('have you tested the below instructions with end-to-end testing and released it') ||
                (clean.length <= 120 && (
                    clean.endsWith('?') ||
                    clean.includes('done') ||
                    clean.includes('published') ||
                    clean.includes('finished') ||
                    clean.includes('released') ||
                    clean.includes('completed')
                ))
            );
        };

        if (typeof promptOrContext === 'object' && promptOrContext !== null) {
            originalPrompt = promptOrContext.originalPrompt || promptOrContext.prompt || '';
            currentRound = promptOrContext.round || round || 1;
            totalRounds = promptOrContext.total || total || 1;
            customNudge = promptOrContext.customNudgeMessage || '';
        } else {
            const rawStr = typeof promptOrContext === 'string' ? promptOrContext.trim() : '';
            currentRound = round ?? 1;
            totalRounds = total ?? 1;

            if (isNudgeText(rawStr)) {
                customNudge = rawStr;
                originalPrompt = '';
            } else {
                originalPrompt = rawStr;
            }
        }

        // Clean up recursively nested rerun delimiters and headers
        if (originalPrompt.includes('<<<ORIGINAL_TASK_PROMPT_START>>>')) {
            const startMarker = '<<<ORIGINAL_TASK_PROMPT_START>>>';
            const endMarker = '<<<ORIGINAL_TASK_PROMPT_END>>>';
            const startIdx = originalPrompt.indexOf(startMarker);
            const endIdx = originalPrompt.indexOf(endMarker);
            if (startIdx !== -1 && endIdx > startIdx) {
                originalPrompt = originalPrompt.substring(startIdx + startMarker.length, endIdx).trim();
            }
        } else if (originalPrompt.includes('# 🔄 RERUN EXECUTION NUDGE')) {
            // Strip the nudge header and delimiters instead of erasing the prompt to empty string
            let stripped = originalPrompt;
            // 1. If there is an extracted custom nudge within the header, extract it if customNudge is not set
            const specificNudgeMatch = stripped.match(/>\s*\*\*Specific Nudge:\*\*\s*([^\r\n]+)/i);
            if (specificNudgeMatch && !customNudge) {
                customNudge = specificNudgeMatch[1].trim();
            }
            // 2. Strip the rerun header line
            stripped = stripped.replace(/# 🔄 RERUN EXECUTION NUDGE[^\r\n]*/gi, '');
            // 3. Strip blockquote lines (> ...)
            stripped = stripped.replace(/^>.*$/gm, '');
            // 4. Strip original task prompt label and horizontal rules
            stripped = stripped.replace(/###\s*Original Task Prompt:?/gi, '');
            stripped = stripped.replace(/^---+$/gm, '');
            // 5. Strip trailing status request line
            stripped = stripped.replace(/Please provide an updated status of your progress in this round and complete any remaining work\.?/gi, '');
            stripped = stripped.trim();

            if (stripped) {
                originalPrompt = stripped;
            } else {
                originalPrompt = '';
            }
        }

        // If customNudge is still empty and originalPrompt looks like a nudge message, shift it
        if (!customNudge && originalPrompt && isNudgeText(originalPrompt)) {
            customNudge = originalPrompt.trim();
            originalPrompt = '';
        }

        // If still no custom nudge, pick a lively default randomized completion check
        if (!customNudge) {
            const fallbackNudges = [
                'Is it finished?',
                'Is it released?',
                'Is it completed?',
                'Please verify and finish the remaining items.',
                'Have you tested these below instructions with end-to-end testing?',
                'Have you tested the below instructions with end-to-end testing and released it?',
            ];
            customNudge = fallbackNudges[Math.floor(Math.random() * fallbackNudges.length)];
        }

        const trimmedPrompt = originalPrompt.trim();
        const lines: string[] = [
            `# 🔄 RERUN EXECUTION NUDGE — ROUND ${currentRound} OF ${totalRounds}`,
            '',
            '> [!NOTE]',
            `> This is automated execution nudge **Round ${currentRound} of ${totalRounds}** for the task below.`,
            '>',
            '> - **State Preservation:** Review files already modified and steps already completed. Do NOT duplicate work.',
            '> - **Forward Progress:** Focus immediately on remaining incomplete or unverified action items.',
            '> - **Self-Correction:** If an earlier step resulted in an error, apply the 4-part Root Cause Analysis (RCA) and correct it now.',
        ];

        if (customNudge) {
            lines.push('>', `> **Specific Nudge:** ${customNudge}`);
        }

        // Only include original prompt section when text is actually present
        if (trimmedPrompt) {
            lines.push(
                '',
                '### Original Task Prompt:',
                '---',
                '<<<ORIGINAL_TASK_PROMPT_START>>>',
                trimmedPrompt,
                '<<<ORIGINAL_TASK_PROMPT_END>>>',
                '---'
            );
        }

        lines.push(
            '',
            'Please provide an updated status of your progress in this round and complete any remaining work.'
        );

        return lines.join('\n');
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.buildRerunPrompt',
            triggerAction: 'buildRerunPrompt',
        });
        const fallback =
            typeof promptOrContext === 'object' && promptOrContext !== null
                ? promptOrContext.customNudgeMessage || promptOrContext.originalPrompt || promptOrContext.prompt || ''
                : typeof promptOrContext === 'string'
                  ? promptOrContext
                  : '';
        return fallback;
    }
}

/**
 * Builds a comprehensive 5-Chapter autonomous verification prompt for multi-prompt series
 * querying task completion, test runs, CI/CD status, and release readiness.
 */
export function buildMultiPromptVerificationPrompt(
    prompts: (string | { prompt_text?: string; full_prompt_text?: string; prompt_preview_200w?: string })[],
    convTitle?: string
): string {
    try {
        const promptTexts: string[] = prompts
            .map((p) => {
                if (typeof p === 'string') return p.trim();
                return (p.full_prompt_text || p.prompt_text || p.prompt_preview_200w || '').trim();
            })
            .filter((t) => t.length > 0);

        const titleLabel = convTitle ? ` for conversation "${convTitle}"` : '';

        // Chapter 1: Verification Mandate & Goal
        const chapter1 = [
            '# 🚨 CHAPTER 1: MULTI-PROMPT VERIFICATION MANDATE & COMPLETION AUDIT',
            '',
            '> [!IMPORTANT]',
            `> **Autonomous Verification Directive${titleLabel}:**`,
            `> You are acting as the Lead Verification Auditor. Below is the series of the last ${promptTexts.length} user prompts/instructions.`,
            '> Your task is to perform a rigorous, evidence-based audit verifying that EVERY single directive across these prompts',
            '> has been fully executed, tested, and resolved without orphan TODOs or unresolved errors.',
            '>',
            '> - **Anti-Hallucination Policy:** Inspect real files on disk. Do not claim verification without citing line ranges.',
            '> - **No Silent Errors:** Disclose all edge cases, failed checks, and outstanding items.',
        ].join('\n');

        // Chapter 2: Grounded Inventory of Last N Prompts
        const inventoryItems = promptTexts.map((text, idx) => {
            const preview = text.length > 300 ? `${text.slice(0, 300)}...` : text;
            return `#### Prompt ${idx + 1} of ${promptTexts.length}:\n\`\`\`text\n${preview}\n\`\`\``;
        });

        const chapter2 = [
            '# 📊 CHAPTER 2: GROUNDED INVENTORY OF AUDITED PROMPTS',
            '',
            `The following ${promptTexts.length} prompt instruction(s) were submitted in this series:`,
            '',
            ...inventoryItems,
        ].join('\n\n');

        // Chapter 3: Multi-Dimensional Verification Matrix
        const chapter3 = [
            '# 🔍 CHAPTER 3: MULTI-DIMENSIONAL VERIFICATION MATRIX',
            '',
            'Perform and report on the following 4 verification dimensions:',
            '',
            '### 3.1 Completeness & Feature Verification',
            '- [ ] Are all requirements from every prompt listed in Chapter 2 completely implemented?',
            '- [ ] Are all user-facing interactions functioning properly?',
            '- [ ] Are state transitions, modals, hotkeys, and event handlers operational?',
            '',
            '### 3.2 Pre-flight Quality Gates & Compilation',
            '- [ ] Run and pass `npm run build` or typecheck locally.',
            '- [ ] Run Rust gates if applicable: `cd src-tauri && cargo fmt -- --check` and `cargo clippy --all-targets --all-features`.',
            '- [ ] Zero compiler errors, zero unresolved linter warnings.',
            '',
            '### 3.3 CI/CD & Pipeline Health',
            '- [ ] Check local runner or CI status (`gitmap pe -t` or `gitmap e d`).',
            '- [ ] Ensure test suites and integration tests pass without flake.',
            '',
            '### 3.4 Governance & Coding Standards',
            '- [ ] Strictly relative repository paths used throughout.',
            '- [ ] Positive boolean polarity enforced (no double negatives).',
            '- [ ] All fallible operations routed through AppError / error store (no bare `catch {}`).',
            '- [ ] Releases and changelogs attributed strictly to `@aukgit` (`(Thanks to @aukgit)`).',
        ].join('\n');

        // Chapter 4: Deliverables & Evidence Contract
        const chapter4 = [
            '# 📝 CHAPTER 4: DELIVERABLES & EVIDENCE SCORECARD',
            '',
            'You MUST complete the following verification table with explicit evidence:',
            '',
            '| Dimension | Status | File / Line Reference | Verifiable Evidence |',
            '| :--- | :---: | :--- | :--- |',
            '| All Prompt Directives Met | PASS / FAIL | `src/...` | Verified against all listed prompts |',
            '| Pre-flight Build (`npm run build`) | PASS / FAIL | Terminal Output | Clean build output |',
            '| Rust Pre-flight (`cargo clippy`) | PASS / FAIL | Terminal Output | Zero clippy warnings |',
            '| Coding Guidelines & Error Visibility | PASS / FAIL | `src/...` | No bare catch, errors logged |',
            '| Task State & Subtask Ledger | PASS / FAIL | `.ai-memory/...` | Ledger marked COMPLETED |',
            '',
            '### Concluding Verdict:',
            '- **VERIFICATION_RESULT:** `[PASS | FAIL]`',
            '- **FAILED_ITEMS:** `[None | Describe any unmet items]`',
        ].join('\n');

        // Chapter 5: Autonomous Execution Directive
        const chapter5 = [
            '# ⚡ CHAPTER 5: AUTONOMOUS REMEDIATION DIRECTIVE',
            '',
            '> [!TIP]',
            '> If ANY check in Chapter 3 or 4 fails, do NOT stop or leave it broken.',
            '> Immediately apply the required fixes, re-verify until 100% green, and report the successful final state.',
        ].join('\n');

        return [chapter1, '', chapter2, '', chapter3, '', chapter4, '', chapter5].join('\n');
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.buildMultiPromptVerificationPrompt',
            triggerAction: 'buildMultiPromptVerificationPrompt',
        });
        return prompts.map((p) => typeof p === 'string' ? p : (p.full_prompt_text || p.prompt_text || p.prompt_preview_200w || '')).join('\n\n---\n\n');
    }
}

// ============================================================================
// MULTI-ROUND VERIFICATION ENGINE & WRITING STYLES (Task-02)
// ----------------------------------------------------------------------------
// Implements 4 specialized verification writing styles and multi-round repetition
// (1x, 2x, 3x progressive escalation) to eliminate superficial affirmation.
// ============================================================================

export type VerificationWritingStyle =
    | 'root-cause-auditor'
    | 'socratic-skeptic'
    | 'spec-gatekeeper'
    | 'release-blocker';

export interface MultiRoundVerificationOptions {
    conversationId: string;
    conversationTitle: string;
    instanceId: string;
    selectedTurnNumbers?: number[];
    roundsCount?: 1 | 2 | 3;
    rounds?: 1 | 2 | 3;
    writingStyle?: VerificationWritingStyle;
    style?: VerificationWritingStyle;
    prompts?: string[];
    turns?: string[];
    repoPath?: string;
    customDirectives?: string;
}

export interface WritingStyleDescriptor {
    id: VerificationWritingStyle;
    title: string;
    subtitle: string;
    description: string;
    iconName: string;
    badge: string;
    color: string;
}

export const WRITING_STYLES: ReadonlyArray<WritingStyleDescriptor> = [
    {
        id: 'root-cause-auditor',
        title: 'Root-Cause Auditor',
        subtitle: 'Failure Forensics & Invariants',
        description: 'Demands 4-part RCA, reproduction traces, surgical minimal fixes, and prevention invariants.',
        iconName: 'Search',
        badge: 'RCA',
        color: 'purple',
    },
    {
        id: 'socratic-skeptic',
        title: 'Socratic Skeptic',
        subtitle: "Devil's Advocate & Stress Probe",
        description: 'Aggressively challenges success claims; probes edge cases, race conditions, nulls, and timeouts.',
        iconName: 'Zap',
        badge: 'Adversary',
        color: 'amber',
    },
    {
        id: 'spec-gatekeeper',
        title: 'Spec Gatekeeper',
        subtitle: '1:1 Acceptance Compliance',
        description: 'Demands line-by-line verification against specification checkboxes and verbatim user instructions.',
        iconName: 'CheckSquare',
        badge: 'Gatekeeper',
        color: 'emerald',
    },
    {
        id: 'release-blocker',
        title: 'Release Blocker',
        subtitle: 'Production Readiness & Gates',
        description: 'Enforces 14-location SemVer sync, strict @aukgit attribution, pre-flight checks, and error visibility.',
        iconName: 'ShieldAlert',
        badge: 'Blocker',
        color: 'rose',
    },
];

/**
 * Builds the chapter content for 'root-cause-auditor' style across rounds 1..3
 */
function buildRootCauseAuditorChapters(round: number, totalRounds: number): string {
    if (round === 1) {
        return [
            '## 🚨 VERIFICATION AUDIT: STRICT ROOT-CAUSE ANALYSIS & FAILURE FORENSICS',
            `> **Writing Style:** \`root-cause-auditor\` | **Round:** 1 of ${totalRounds}`,
            '',
            '### Mandatory 4-Part RCA Contract:',
            'Every touched module or resolved defect MUST be audited against the following 4-part schema:',
            '1. **Problem Statement:** Cite exact symptoms, error logs, and stack traces.',
            '2. **Root Cause Analysis:** Explain the exact physical mechanism of failure (unhandled branch, race condition, type coercion).',
            '3. **Surgical Remediation:** Confirm that only minimal, necessary lines were changed.',
            '4. **Architectural Prevention Invariant:** Formulate an explicit invariant preventing regression.',
            '',
            '| Defect / Touched Module | Root Cause Diagnosis | Minimal Surgical Fix | Prevention Invariant | Status |',
            '| :--- | :--- | :--- | :--- | :---: |',
            '| `src/...#L1-L50` | Grounded flaw explanation | Line-level fix citation | Invariant I-X | PASS/FAIL |',
            '',
            '### Physical Source File Inspections:',
            '- Verify that every file cited was actually inspected on disk.',
            '- Zero speculative assumptions; cite real file paths relative to repo root (`src/...`, `src-tauri/...`).',
        ].join('\n');
    }

    if (round === 2) {
        return [
            '## 🔬 DEEP FORENSIC PROBE: REPRODUCTION TRACE & REGRESSION INVARIANTS',
            `> **Writing Style:** \`root-cause-auditor\` | **Round:** 2 of ${totalRounds}`,
            '',
            '### Reproduction & Flaw Isolation:',
            '1. **Reproduction Mechanism:** State the exact physical CLI command, payload, or unit test that triggers the defect before fix.',
            '2. **Secondary Regression Check:** Did the surgical fix perturb any adjacent data structures, schemas, or IPC calls?',
            '3. **No Symptom Masking:** Verify that error handling does not suppress root errors (`catch {}` or silent `.ok()` are forbidden).',
            '',
            '| Module Tested | Reproduction Trace | Regression Guard | Physical Proof | Verdict |',
            '| :--- | :--- | :--- | :--- | :---: |',
            '| `src/...` | Bounded repro trace | Tested against regression | Unit test / CLI output | VERIFIED |',
        ].join('\n');
    }

    return [
        '## 🏁 FORENSIC SIGN-OFF: PREVENTION INVARIANT ENFORCEMENT & ARCHITECTURAL IMMUNITY',
        `> **Writing Style:** \`root-cause-auditor\` | **Round:** 3 of ${totalRounds}`,
        '',
        '### Long-Term Immunity Invariant:',
        '1. **Structural Prevention:** Is the fix architecturally enforced so the entire class of problem is physically impossible in future code?',
        '2. **Memory & Plan Synchronization:** Are all plans under `.ai-memory/plans/` updated and marked COMPLETED?',
        '3. **Audit Ledger Verification:** Confirm root-cause evidence is recorded in SQLite task history or agent-task.db.',
        '',
        '| Invariant ID | Rule Definition | Automated Test / Gate | Final Sign-off |',
        '| :--- | :--- | :--- | :---: |',
        '| INV-ROOT-01 | Invariant description | Linter / compiler / test | CERTIFIED |',
    ].join('\n');
}

/**
 * Builds the chapter content for 'socratic-skeptic' style across rounds 1..3
 */
function buildSocraticSkepticChapters(round: number, totalRounds: number): string {
    if (round === 1) {
        return [
            '## ⚡ ADVERSARIAL VERIFICATION: SOCRATIC SKEPTIC AUDIT',
            `> **Writing Style:** \`socratic-skeptic\` | **Round:** 1 of ${totalRounds}`,
            '',
            '### Skeptic Stance & Null-Hypothesis:',
            'Do NOT affirm success. Treat every claim of completion as guilty until proven innocent:',
            '1. **Unsubstantiated Claims:** What claims were made without providing terminal output or file citations?',
            '2. **Assumption Auditing:** What assumptions did the implementation rely on that were never explicitly verified?',
            '3. **Failure Visibility:** Are errors swallowed or hidden behind fake success toasts?',
            '',
            '| Claimed Feature | Purported Proof | Socratic Challenge | Verified Reality | Verdict |',
            '| :--- | :--- | :--- | :--- | :---: |',
            '| Feature Implementation | "Works cleanly" | Was edge case tested? | Physical file check | SCRUTINIZED |',
        ].join('\n');
    }

    if (round === 2) {
        return [
            '## 🌪️ BOUNDARY STRESS PROBE: RACE CONDITIONS, NULLS & FAULT INJECTION',
            `> **Writing Style:** \`socratic-skeptic\` | **Round:** 2 of ${totalRounds}`,
            '',
            '### Adversarial Stress Scenarios:',
            '1. **Boundary Values:** How does the system handle 0-length strings, negative numbers, 10MB payloads, or emoji floods?',
            '2. **Concurrency Hazards:** What happens if the user rapidly double-clicks dispatch within 50ms? Are debounces or locks active?',
            '3. **Process & Storage Faults:** What happens if SQLite is locked (`SQLITE_BUSY`), the filesystem is read-only, or network sockets drop?',
            '',
            '| Adversarial Scenario | Stress Mechanism | Expected Safe Behavior | Physical Evidence | Verdict |',
            '| :--- | :--- | :--- | :--- | :---: |',
            '| Empty / Malformed Payload | `prompts = []` | Graceful error toast | Error modal / AppError | IMMUNE |',
            '| Rapid Double Click | Clicks spaced 30ms | Guard locks second action | Button disabled / loading | IMMUNE |',
            '| File Write Collision | Simultaneous rounds | 500ms safety sleep | FIFO queue sequencing | IMMUNE |',
        ].join('\n');
    }

    return [
        '## 🛡️ RESILIENCE SIGN-OFF: GRACEFUL DEGRADATION & IMMUNITY CERTIFICATION',
        `> **Writing Style:** \`socratic-skeptic\` | **Round:** 3 of ${totalRounds}`,
        '',
        '### Adversarial Resilience Verdict:',
        '1. Has the implementation withstood all stress probes without throwing unhandled exceptions or crashing?',
        '2. Are all error cases routed through `AppError` (backend) and `useErrorStore` (frontend)?',
        '3. Can this code be safely exposed to malicious or erratic inputs in production?',
        '',
        '| Stress Category | Maximum Tolerance Observed | Recovery Mechanism | Resilience Sign-off |',
        '| :--- | :--- | :--- | :---: |',
        '| Boundary Inputs | 0 to 100k chars | Guard clamp / graceful return | CERTIFIED |',
        '| Concurrency Races | 50 concurrent requests | Mutex / FIFO queue serialized | CERTIFIED |',
    ].join('\n');
}

/**
 * Builds the chapter content for 'spec-gatekeeper' style across rounds 1..3
 */
function buildSpecGatekeeperChapters(round: number, totalRounds: number): string {
    if (round === 1) {
        return [
            '## 📋 SPECIFICATION GATEKEEPER: 1:1 ACCEPTANCE CRITERIA AUDIT',
            `> **Writing Style:** \`spec-gatekeeper\` | **Round:** 1 of ${totalRounds}`,
            '',
            '### Strict Specification Verification Contract:',
            'Audit the implemented code against every line item of the parent specification:',
            '- Every `- [ ]` checkbox must be either verified as `- [x]` with file citations or reported as FAILED.',
            '- All UI component names, props, and Tauri IPC command signatures must match specification types verbatim.',
            '',
            '| Spec Item ID | Specification Directive | Physical Code Location | Line-by-Line Evidence | Compliance |',
            '| :--- | :--- | :--- | :--- | :---: |',
            '| SPEC-2.1 | 4 Writing Styles Defined | `verificationTemplates.ts` | Lines 50-120 | 100% PASS |',
            '| SPEC-2.2 | 500ms Safety Throttling | `verification.rs` | Lines 85-92 | 100% PASS |',
        ].join('\n');
    }

    if (round === 2) {
        return [
            '## 🔍 NEGATIVE SPECIFICATION AUDIT: BOUNDARY CONSTRAINTS & BANNED PATTERNS',
            `> **Writing Style:** \`spec-gatekeeper\` | **Round:** 2 of ${totalRounds}`,
            '',
            '### Negative Specification Directives:',
            '1. **Zero Git Commands:** Verify that no prohibited git commands (`git add`, `git commit`, `git push`, `git status`) were executed.',
            '2. **Search Discipline:** Verify that search strictly used `gitmap` and zero banned tools (`grep`, `rg`, `findstr`).',
            '3. **Disjoint Ownership:** Verify that only assigned owned files were edited; zero collateral changes.',
            '4. **Strict Relative Paths:** All paths must be relative (`src/...`, `src-tauri/...`), never machine-absolute.',
            '',
            '| Guard Rule | Restriction Scope | Compliance Audit | Status |',
            '| :--- | :--- | :--- | :---: |',
            '| Git Command Ban | Whole session | Zero git calls in transcript | COMPLIANT |',
            '| Search Tool Ban | Whole session | Gitmap used exclusively | COMPLIANT |',
            '| Relative Paths | All files & templates | Zero absolute paths found | COMPLIANT |',
        ].join('\n');
    }

    return [
        '## 🏆 SPECIFICATION CERTIFICATION: 100% DELIVERABLE SIGN-OFF',
        `> **Writing Style:** \`spec-gatekeeper\` | **Round:** 3 of ${totalRounds}`,
        '',
        '### Acceptance Certification:',
        '- Are all parent spec requirements (`02-spec/...`) satisfied with zero orphaned items?',
        '- Is the subtask ledger in `.ai-memory/temp-agents/...` updated to COMPLETED?',
        '- Final Compliance Verdict: [100% PASS / REJECTED]',
        '',
        '| Milestone | Deliverable Scope | Completion Evidence | Final Status |',
        '| :--- | :--- | :--- | :---: |',
        '| Subtask 02 Deliverables | Engine, Styles, Modal, Backend | All 5 files verified | CERTIFIED 100% |',
    ].join('\n');
}

/**
 * Builds the chapter content for 'release-blocker' style across rounds 1..3
 */
function buildReleaseBlockerChapters(round: number, totalRounds: number): string {
    if (round === 1) {
        return [
            '## 🛑 PRODUCTION RELEASE BLOCKER: GOVERNANCE & QUALITY GATES INSPECTION',
            `> **Writing Style:** \`release-blocker\` | **Round:** 1 of ${totalRounds}`,
            '',
            '### Non-Negotiable Release Gates:',
            'Before certifying release readiness, the following gates MUST pass without exception:',
            '1. **Atomic Version Sync:** Check all 14 version manifest locations for `vX.Y.Z` parity.',
            '2. **Strict Attribution Invariant:** Verify that ONLY `@aukgit` is credited in changelogs.',
            '3. **Pre-flight Quality Gates:** Run `cargo fmt`, `cargo clippy`, and `npm run build`.',
            '4. **Error Visibility Invariant:** Confirm all fallible operations route into `AppError` or `useErrorStore`.',
            '',
            '| Release Gate | Verification Command | Gate Criteria | Result | Blocker Status |',
            '| :--- | :--- | :--- | :---: | :---: |',
            '| Rust Formatting | `cargo fmt -- --check` | Exit code 0 | PASS | CLEAR |',
            '| Rust Clippy | `cargo clippy --all-features` | Zero warnings | PASS | CLEAR |',
            '| Frontend Build | `npm run build` | Clean bundle, zero TS errors | PASS | CLEAR |',
            '| Error Visibility | Static inspection | Zero bare `catch {}` | PASS | CLEAR |',
        ].join('\n');
    }

    if (round === 2) {
        return [
            '## 🚨 DEEP GOVERNANCE AUDIT: ATTRIBUTION, CHANGELOGS & CHANNEL SEPARATION',
            `> **Writing Style:** \`release-blocker\` | **Round:** 2 of ${totalRounds}`,
            '',
            '### Release Channel & Attribution Integrity:',
            '1. **Channel Separation:** Confirm branch aligns with release channel (`main` for production, `beta` for preview).',
            '2. **Exclusive Attribution:** Verify `changelog.md` and `changelog_en.md` attribute strictly `(Thanks to @aukgit)`. Zero other handles allowed.',
            '3. **README Sync:** Ensure root `README.md` ("## 📝 Changelog") and `README_EN.md` ("## 📝 Changelog") are in sync.',
            '4. **Thinking Cache Toggle:** Verify `SUGGESTION_DELETE_THINKING_STORE` in `SuggestionDeleteThinkingModal.tsx`.',
            '',
            '| Governance Item | Expected Invariant | Verified Reality | Status |',
            '| :--- | :--- | :--- | :---: |',
            '| Attribution | `@aukgit` exclusive | Verified via changelog | PASS |',
            '| README Sync | README + README_EN matching | Verified update logs | PASS |',
            '| Channel Tag | `vX.Y.Z` matching heading | Tag gate alignment | PASS |',
        ].join('\n');
    }

    return [
        '## 🚀 FINAL RELEASE CLEARANCE: PRODUCTION DEPLOYMENT PERMISSION',
        `> **Writing Style:** \`release-blocker\` | **Round:** 3 of ${totalRounds}`,
        '',
        '### Final Release Gate Verdict:',
        '- **ALL_GATES_PASSED:** `[TRUE | FALSE]`',
        '- **BLOCKING_DEFECTS:** `[None | Describe blocking issues]`',
        '- **RELEASE_AUTHORIZATION:** `[AUTHORIZED | BLOCKED]`',
        '',
        '| Release Checkpoint | Criteria | Verification Evidence | Sign-off |',
        '| :--- | :--- | :--- | :---: |',
        '| SemVer Atomic Sync | 14 manifests identical | Validated via bump script | APPROVED |',
        '| Strict @aukgit | Exclusively aukgit in logs | Verified zero other tags | APPROVED |',
        '| Clean Compiles | cargo clippy + npm build | Exit code 0 | APPROVED |',
    ].join('\n');
}

/**
 * Builds structured multi-round verification prompts.
 * Generates an array of N Markdown prompts for rounds 1..N.
 *
 * Supports both signatures:
 * - buildMultiRoundVerificationPrompts(options)
 * - buildMultiRoundVerificationPrompts(turns, options)
 */
export function buildMultiRoundVerificationPrompts(
    turnsOrOptions: string[] | MultiRoundVerificationOptions,
    maybeOptions?: Partial<MultiRoundVerificationOptions>
): string[] {
    try {
        let prompts: string[] = [];
        let opts: Partial<MultiRoundVerificationOptions> = {};

        if (Array.isArray(turnsOrOptions)) {
            prompts = turnsOrOptions;
            opts = maybeOptions || {};
        } else if (typeof turnsOrOptions === 'object' && turnsOrOptions !== null) {
            opts = turnsOrOptions;
            prompts = turnsOrOptions.prompts || turnsOrOptions.turns || [];
        }

        const rawStyle = opts.writingStyle || opts.style || 'root-cause-auditor';
        const style: VerificationWritingStyle =
            rawStyle === 'socratic-skeptic' ||
            rawStyle === 'spec-gatekeeper' ||
            rawStyle === 'release-blocker'
                ? rawStyle
                : 'root-cause-auditor';

        const rawRounds = opts.roundsCount ?? opts.rounds ?? 1;
        const totalRounds = Math.max(1, Math.min(3, Math.floor(rawRounds))) as 1 | 2 | 3;
        const conversationId = opts.conversationId || 'default-session';
        const conversationTitle = opts.conversationTitle || conversationId || 'Current Task';
        const customDirectives = (opts.customDirectives || '').trim();

        const cleanPrompts = prompts
            .map((p) => (typeof p === 'string' ? p.trim() : ''))
            .filter((p) => p.length > 0);

        const results: string[] = [];

        for (let r = 1; r <= totalRounds; r++) {
            const banner = [
                `# 🛡️ VERIFICATION ROUND ${r} OF ${totalRounds}: [${style.toUpperCase()}]`,
                '',
                '> [!IMPORTANT]',
                `> **Verification Escalation Directive (Round ${r} of ${totalRounds}):**`,
                `> You are executing Round ${r} of an automated ${totalRounds}-round verification cycle.`,
                `> Conversation: **${conversationTitle}** (\`${conversationId}\`)`,
                r === 1 ? '> **Round 1 Focus:** Baseline audit, directive compliance, and direct requirement verification.' : '',
                r === 2 ? '> **Round 2 Focus:** Adversarial edge cases, stress testing, concurrency hazards, and regression verification.' : '',
                r === 3 ? '> **Round 3 Focus:** Production sign-off, SemVer parity, @aukgit attribution, and quality gate clearance.' : '',
                '> Focus your cognitive effort strictly on the designated depth for this round.',
            ].filter(Boolean).join('\n');

            let styleChapters = '';
            switch (style) {
                case 'root-cause-auditor':
                    styleChapters = buildRootCauseAuditorChapters(r, totalRounds);
                    break;
                case 'socratic-skeptic':
                    styleChapters = buildSocraticSkepticChapters(r, totalRounds);
                    break;
                case 'spec-gatekeeper':
                    styleChapters = buildSpecGatekeeperChapters(r, totalRounds);
                    break;
                case 'release-blocker':
                    styleChapters = buildReleaseBlockerChapters(r, totalRounds);
                    break;
            }

            const customSection = customDirectives
                ? [
                      '',
                      '### Operator Custom Directives:',
                      `> ${customDirectives}`,
                  ].join('\n')
                : '';

            const promptInventory = [
                '### Audited Instruction Turns:',
                '<<<AUDITED_USER_PROMPT_START>>>',
                cleanPrompts.length > 0
                    ? cleanPrompts.map((p, idx) => `[Turn ${idx + 1}]:\n${p}`).join('\n\n---\n\n')
                    : 'Verify all completed operations and code modifications for this conversation.',
                '<<<AUDITED_USER_PROMPT_END>>>',
            ].join('\n');

            const conclusion = [
                '### Concluding Evidence Contract:',
                'Complete the verification tables above with physical line-numbered file references and verifiable outputs.',
                'If any check fails, immediately fix the underlying code, re-run tests until green, and report the successful final state.',
            ].join('\n');

            results.push(
                [banner, '', styleChapters, customSection, '', promptInventory, '', conclusion]
                    .filter((s) => s.length > 0)
                    .join('\n')
            );
        }

        return results;
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.buildMultiRoundVerificationPrompts',
            triggerAction: 'buildMultiRoundVerificationPrompts',
        });
        const fallback = Array.isArray(turnsOrOptions)
            ? turnsOrOptions
            : (turnsOrOptions && (turnsOrOptions.prompts || turnsOrOptions.turns)) || [];
        return [fallback.join('\n\n---\n\n')];
    }
}


