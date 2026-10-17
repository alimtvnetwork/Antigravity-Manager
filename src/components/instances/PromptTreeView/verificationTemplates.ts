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
    | 'are-you-confident'
    | 'edge-cases-and-tests'
    | 'security-audit'
    | 'pr-review-ready'
    | 'test-and-coverage'
    | 'refactor-dry'
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
    originalPrompt: string;
    round: number;
    total: number;
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
                '- [ ] **4. README Synchronization:** Are both `README.md` (under "## 📝 更新日志") and `README_EN.md` (under "## 📝 Changelog") update logs synchronized with the release notes?',
                '- [ ] **5. Pre-flight Quality Gates:** Have `cd src-tauri && cargo fmt -- --check`, `cd src-tauri && cargo clippy --all-targets --all-features`, and `npm run build` all passed cleanly?',
                '- [ ] **6. Git Tag Precision:** Does the git tag strictly match `CHANGELOG.md` heading character-for-character (`vX.Y.Z` on main, `vX.Y.Z-beta.N` on beta)?',
                '- [ ] **7. Distribution Scripts:** Are `install.ps1` and `install.sh` verified for the target release?',
                '- [ ] **8. Thinking Store Invalidation Toggle:** Is `SUGGESTION_DELETE_THINKING_STORE` correctly configured in `SuggestionDeleteThinkingModal.tsx`?',
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

        if (typeof promptOrContext === 'object' && promptOrContext !== null) {
            originalPrompt = promptOrContext.originalPrompt || '';
            currentRound = promptOrContext.round || 1;
            totalRounds = promptOrContext.total || 1;
            customNudge = promptOrContext.customNudgeMessage || '';
        } else {
            originalPrompt = typeof promptOrContext === 'string' ? promptOrContext : '';
            currentRound = round ?? 1;
            totalRounds = total ?? 1;
        }

        const customNudgeSection = customNudge
            ? `\n> **Specific Nudge:** ${customNudge}\n`
            : '';

        return [
            `# 🔄 RERUN EXECUTION NUDGE — ROUND ${currentRound} OF ${totalRounds}`,
            '',
            '> [!NOTE]',
            `> This is automated execution nudge **Round ${currentRound} of ${totalRounds}** for the task below.`,
            '>',
            '> - **State Preservation:** Review files already modified and steps already completed. Do NOT duplicate work.',
            '> - **Forward Progress:** Focus immediately on remaining incomplete or unverified action items.',
            '> - **Self-Correction:** If an earlier step resulted in an error, apply the 4-part Root Cause Analysis (RCA) and correct it now.',
            customNudgeSection,
            '### Original Task Prompt:',
            '---',
            '<<<ORIGINAL_TASK_PROMPT_START>>>',
            originalPrompt.trim(),
            '<<<ORIGINAL_TASK_PROMPT_END>>>',
            '---',
            '',
            'Please provide an updated status of your progress in this round and complete any remaining work.',
        ].join('\n');
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.buildRerunPrompt',
            triggerAction: 'buildRerunPrompt',
        });
        const fallback =
            typeof promptOrContext === 'object' && promptOrContext !== null
                ? promptOrContext.originalPrompt || ''
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

