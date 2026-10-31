# Component Specification: Discrete Guard Clauses & Boolean Helper Extraction

- **Specification Slug:** `04-booleans-and-complex-conditions`
- **Specification Path:** `02-spec/21-app/04-booleans-and-complex-conditions/02-component-spec.md`
- **Target Release Version:** `v4.244.0` (Minor Bump from `v4.243.0`)
- **Author:** `@aukgit`
- **Status:** APPROVED & MANDATED

---

## 1. `isProjectRunningForInstance` Refactoring Specification

### 1.1 Existing Anti-Pattern
```typescript
// ❌ FORBIDDEN: Compound negative chain and 4-operand mixed condition
export function isProjectRunningForInstance(proj: AgmProjectTreeNode, inst: InstanceStatus): boolean {
    if (!inst.is_running || !proj.is_running) return false;
    return (proj.conversations || []).some(
        (c) =>
            Boolean(c.is_running) &&
            isConversationOwnedByInstance(c.instance_id, inst.config, c.instance_seq_num ?? proj.instance_seq_num) &&
            !isGhostConversation(c) &&
            !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
    );
}
```

### 1.2 Target Refactored Structure
```typescript
// ✅ REQUIRED: Discrete early returns, affirmative named helper, max 2 operands
export function hasMeaningfulPrompt(conv: AgmConversationNode): boolean {
    if (conv.prompt_word_count > 0) {
        return true;
    }
    const preview = (conv.prompt_preview_200w || '').trim();
    return preview.length > 0;
}

export function isConversationActiveForInstance(
    conv: AgmConversationNode,
    instConfig: InstanceStatus['config'],
    defaultSeqNum?: number
): boolean {
    if (!conv.is_running) {
        return false;
    }
    const isOwned = isConversationOwnedByInstance(
        conv.instance_id,
        instConfig,
        conv.instance_seq_num ?? defaultSeqNum
    );
    if (!isOwned) {
        return false;
    }
    if (isGhostConversation(conv)) {
        return false;
    }
    return hasMeaningfulPrompt(conv);
}

export function isProjectRunningForInstance(proj: AgmProjectTreeNode, inst: InstanceStatus): boolean {
    if (!inst.is_running) {
        return false;
    }
    if (!proj.is_running) {
        return false;
    }
    const conversations = proj.conversations || [];
    return conversations.some((conv) =>
        isConversationActiveForInstance(conv, inst.config, proj.instance_seq_num)
    );
}
```

---

## 2. Benefits
1. **Zero Compound Negatives**: Eliminates `!a || !b`.
2. **Two-Condition Max**: Every single expression contains at most 2 operands.
3. **No Mixed Logical Operators**: Zero expressions mix `&&` with `||`.
4. **Affirmative Extraction**: Replaces inverted De Morgan's logic with `hasMeaningfulPrompt`.
