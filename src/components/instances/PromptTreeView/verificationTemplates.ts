/**
 * Verification & Rerun Prompt Generation Engine
 *
 * Implements the 5-Chapter prompt architecture for task completion audits,
 * release engineering verification, architectural confidence assessments,
 * and iteration-tracked rerun nudges.
 */

import { useErrorStore } from '../../../stores/error-store';

/**
 * Supported verification template types.
 */
export type VerificationTemplateType =
    | 'is-it-done'
    | 'is-it-released'
    | 'are-you-confident'
    | 'edge-cases'
    | 'edge-cases-and-tests'
    | 'verify-tests'
    | 'rerun-nudge'
    | 'none';

/**
 * Metadata descriptor for verification options rendered in UI dropdowns.
 */
export interface VerificationOptionMeta {
    value: VerificationTemplateType;
    label: string;
    dropdownLabel?: string;
    badgeLabel?: string;
    description?: string;
    iconName?: 'check-circle' | 'rocket' | 'shield-check' | 'alert-triangle' | 'terminal' | 'repeat' | 'slash';
}

/**
 * Context supplied when generating a rich verification prompt.
 */
export interface VerificationTemplateContext {
    /** The complete original prompt text to embed into Chapter 4. */
    originalPrompt: string;
    /** Template category / kind discriminator. */
    kind?: string;
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
 * All available verification options for UI selection in DetailHeader.
 */
export const VERIFICATION_OPTIONS: ReadonlyArray<VerificationOptionMeta> = [
    {
        value: 'none',
        label: 'None (Send as is)',
        dropdownLabel: 'None',
        badgeLabel: 'Default',
        description: 'Send prompt without verification envelope',
        iconName: 'slash',
    },
    {
        value: 'is-it-done',
        label: 'Is it done? (Audit Checklist)',
        dropdownLabel: 'Done?',
        badgeLabel: 'Done',
        description: 'Comprehensive completion audit & evidence scorecard',
        iconName: 'check-circle',
    },
    {
        value: 'is-it-released',
        label: 'Is it released? (Release Gates)',
        dropdownLabel: 'Released?',
        badgeLabel: 'Release',
        description: 'Release channel, 14-location SemVer, attribution & gate checks',
        iconName: 'rocket',
    },
    {
        value: 'are-you-confident',
        label: 'Are you confident? (Quality Audit)',
        dropdownLabel: 'Sure?',
        badgeLabel: 'Quality',
        description: 'Code quality, <= 15-line functions, positive booleans & error routing',
        iconName: 'shield-check',
    },
    {
        value: 'edge-cases-and-tests',
        label: 'Edge Cases & Tests',
        dropdownLabel: 'Edge cases',
        badgeLabel: 'Tests',
        description: 'Boundary checks, concurrency, recovery ladders & test suite gates',
        iconName: 'alert-triangle',
    },
];

/**
 * Default option when none is explicitly selected.
 */
export const DEFAULT_VERIFICATION_TYPE: VerificationTemplateType = 'none';

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
        normalized.includes('done')
    ) {
        return 'is-it-done';
    }

    if (
        normalized === 'is-it-released' ||
        normalized === 'is it released?' ||
        normalized === 'released?' ||
        normalized.includes('release')
    ) {
        return 'is-it-released';
    }

    if (
        normalized === 'are-you-confident' ||
        normalized === 'are you sure about it?' ||
        normalized === 'sure?' ||
        normalized.includes('confident') ||
        normalized.includes('sure')
    ) {
        return 'are-you-confident';
    }

    if (
        normalized === 'edge-cases-and-tests' ||
        normalized === 'edge-cases' ||
        normalized === 'double check all edge cases' ||
        normalized.includes('edge')
    ) {
        return 'edge-cases-and-tests';
    }

    if (
        normalized === 'verify-tests' ||
        normalized === 'verify build and tests' ||
        normalized.includes('test') ||
        normalized.includes('build')
    ) {
        return 'edge-cases-and-tests';
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
function buildChapter2Checklist(type: VerificationTemplateType): string {
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
        case 'verify-tests':
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
function buildChapter4OriginalPrompt(originalPrompt: string): string {
    const trimmed = (originalPrompt || '').trim();
    return [
        '# 📋 CHAPTER 4: VERBATIM ORIGINAL PROMPT & TASK SPECIFICATION',
        '',
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
              currentRound?: number;
              totalRounds?: number;
          },
    context?: VerificationTemplateContext
): string {
    try {
        let rawKind = 'is-it-done';
        let originalPrompt = '';

        if (typeof optionsOrType === 'object' && optionsOrType !== null) {
            rawKind = optionsOrType.kind || optionsOrType.type || 'is-it-done';
            originalPrompt = optionsOrType.originalPrompt || '';
        } else if (typeof optionsOrType === 'string') {
            rawKind = optionsOrType;
            originalPrompt = context?.originalPrompt || '';
        }

        const templateType = parseVerificationTypeFromSuffix(rawKind);
        if (templateType === 'none') {
            return originalPrompt;
        }

        return [
            buildChapter1AuditMandate(),
            '',
            buildChapter2Checklist(templateType),
            '',
            buildChapter3TelemetryGates(),
            '',
            buildChapter4OriginalPrompt(originalPrompt),
            '',
            buildChapter5EvidenceContract(),
        ].join('\n');
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'verificationTemplates.buildRichVerificationPrompt',
            triggerAction: 'buildRichVerificationPrompt',
        });
        const fallbackPrompt = typeof optionsOrType === 'object' && optionsOrType !== null
            ? optionsOrType.originalPrompt || ''
            : context?.originalPrompt || '';
        return fallbackPrompt;
    }
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
        const fallback = typeof promptOrContext === 'object' && promptOrContext !== null
            ? promptOrContext.originalPrompt || ''
            : typeof promptOrContext === 'string'
              ? promptOrContext
              : '';
        return fallback;
    }
}
