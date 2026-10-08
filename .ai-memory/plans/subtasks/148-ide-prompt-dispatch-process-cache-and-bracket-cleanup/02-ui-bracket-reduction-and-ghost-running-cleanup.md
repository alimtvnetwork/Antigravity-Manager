# Subtask 02: UI Bracket Reduction & Double-Launch Race Removal

- **Subtask ID**: `148-02`
- **Parent Task**: `148-ide-prompt-dispatch-process-cache-and-bracket-cleanup`
- **Target Files**:
  - `src/components/instances/PromptTreeViewModal.tsx`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: READY_FOR_EXECUTION

---

## 1. Objective

Refactor `src/components/instances/PromptTreeViewModal.tsx` to completely eradicate bracket tag clutter (`[AGM:P006 | GM:#6]`, `[AGM:C025 | GM:antigrav]`, `[Omitted ...]`), replace `formatDualBadge` output with clean `#6`, `P006`, and `C025` sequence indicators, update context omission banners to `⚡ Omitted {formattedSize} transcript context · Click to expand`, clean up the header sequence display without double `#` prefixes, and eliminate the frontend double-launch race condition.

---

## 2. Context & Requirements

1. **Heavy Bracket Clutter**:
   - In previous builds, every project row rendered `[AGM:P006 | GM:#6]` and every conversation node rendered `[AGM:C025 | GM:antigrav]`. These bulky wrappers overwhelmed horizontal line space and introduced visual noise.
2. **Double Prefix in Detail Header**:
   - The prompt preview pane rendered `#{selectedConversation.seq_code || 'P001'}`. If the sequence code was already `C025` or `#C025`, it rendered as `##C025` or `#C025` with redundant formatting.
3. **Bracketed Omission Text**:
   - Lines 226 and 605 rendered `<span>[Omitted {formattedSize} of transcript context - Click to inspect/expand]</span>`. The specification mandates clean interactive dark-glass omission banners: `⚡ Omitted {formattedSize} transcript context · Click to expand` without outer brackets.
4. **Double-Launch Race Condition**:
   - In `handleResendPrompt` (~L1703-1718), invoking `sendPromptNow` immediately followed by `focusOrLaunchInstance` triggered two competing IDE launch sequences, resulting in duplicate windows.

---

## 3. Detailed Implementation Steps

### Step 1: Refactor `formatDualBadge` in `src/components/instances/PromptTreeViewModal.tsx`
- Locate `formatDualBadge` (~L118-123).
- Update the regex and logic to strip bracket tokens, `AGM:`, and `GM:` prefixes:
  ```typescript
  function formatDualBadge(agmCode: string | undefined, defaultAgm: string, gmCode: string | undefined, defaultGm: string): string {
      const raw = agmCode || defaultAgm || gmCode || defaultGm;
      const clean = raw.replace(/^(AGM:|GM:)/i, '').replace(/[\[\]]/g, '').trim();
      return clean.startsWith('#') || clean.startsWith('P') || clean.startsWith('C') ? clean : `#${clean}`;
  }
  ```
- Ensure project badges render as `#6` or `P006`, and conversation badges render as `C025`.

### Step 2: Fix Header Sequence Code Rendering
- Locate the sequence badge inside the prompt content header (~L2886-2888):
  ```tsx
  <span className="inline-flex items-center px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono font-bold bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20">
      {(() => {
          const raw = selectedConversation.seq_code || 'P001';
          const clean = raw.replace(/^(AGM:|GM:)/i, '').replace(/[\[\]]/g, '').trim();
          return clean.startsWith('#') || clean.startsWith('P') || clean.startsWith('C') ? clean : `#${clean}`;
      })()}
  </span>
  ```
- Ensure no duplicate `#` is prepended to `C025` or `P006`.

### Step 3: Upgrade Context Omission Banners
- Locate omission banner spans (~L226 and ~L605) and replace bracketed text with the dark-glass interactive omission callout:
  - Text pattern: `⚡ Omitted {formattedSize} transcript context · Click to expand`
  - Remove all outer square brackets `[` and `]`.
  - Maintain full click-to-expand / toggle functionality.

### Step 4: Eradicate Double-Launch Race in `handleResendPrompt`
- Locate `handleResendPrompt` (~L1690-1727).
- Remove the redundant call to `await focusOrLaunchInstance(targetInstId, repoPath);` immediately following `sendPromptNow`.
- Keep clipboard copy (`navigator.clipboard.writeText(promptContent)`) as user convenience.
- Allow backend `dispatch_prompt_now` to handle smart instance detection, PID re-scan, launch, and workspace window focusing.

---

## 4. Invariants & Constraints

- **Strict Relative Git Paths**: All paths cited in code and documentation must be relative (e.g. `src/components/instances/PromptTreeViewModal.tsx`).
- **Positive Booleans**: Exclusively use positive boolean identifiers (`is_resending`, `is_context_expanded`, `is_running`).
- **Zero Content Truncation**: Preserve original multiline formatting and clipboard fidelity.
- **Cross-Platform Compatibility**: Ensure clipboard and modal interactions work seamlessly across Linux, macOS, and Windows.

---

## 5. Verification & Pre-flight Checklist

```bash
# Verify TypeScript syntax and frontend bundling
npm run build
```

---

## 6. Done When

- [ ] All `[AGM:... | GM:...]` brackets are removed from project rows and conversation nodes in `PromptTreeViewModal.tsx`.
- [ ] Clean sequence codes `#6`, `P006`, and `C025` are displayed.
- [ ] Header detail pane renders clean sequence badges without `#C025` double prefix.
- [ ] Context omission banners render `⚡ Omitted {formattedSize} transcript context · Click to expand` without brackets.
- [ ] Duplicate `focusOrLaunchInstance` call is removed from `handleResendPrompt`, preventing double window launches.
- [ ] `npm run build` succeeds with zero errors.
