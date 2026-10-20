# System Architecture & Technical Specification: Boolean Logic Modernization & Guard Clause Decomposition

- **Specification Slug:** `04-booleans-and-complex-conditions`
- **Specification Path:** `02-spec/21-app/04-booleans-and-complex-conditions/01-architecture-spec.md`
- **Target Release Version:** `v4.244.0` (Minor Bump from `v4.243.0`)
- **Author:** `@aukgit`
- **Status:** APPROVED & MANDATED

---

## 1. User Request (Verbatim)

```text
Try to have better versions of the if  else please read and apply
@[01-prompts/15-cg-execute/04-booleans-and-complex-conditions.md] 

and release with a minor bump please

if (!inst.is_running || !proj.is_running) return false;
```

---

## 2. Architectural Pillars & Core Principles

### 2.1 The Two-Condition Maximum Rule (Principle P4a)
A single boolean expression may combine at most two operands with `&&` or `||`. Three or more operands must be decomposed into intermediate named booleans or discrete early-return guard clauses.

### 2.2 Strict Operator Isolation (Principle P4b)
A single boolean expression must use only one logical operator type (`&&` or `||`). Mixing `&&` and `||` in the same expression creates precedence ambiguity, introduces subtle boolean bugs, and increases cognitive overhead.

### 2.3 Elimination of Compound Negative Chains
Compound negative expressions such as `!a || !b` force readers to mentally compute De Morgan's laws. Such chains are replaced with discrete, early-return guards:
```typescript
// ❌ FORBIDDEN: Compound negative chain
if (!inst.is_running || !proj.is_running) return false;

// ✅ REQUIRED: Discrete early returns with affirmative naming
if (!inst.is_running) {
    return false;
}
if (!proj.is_running) {
    return false;
}
```

### 2.4 Separation of Positive and Negative Checks (Principle P4c)
Never combine a negative check (`!`, `null`, `undefined`) with positive checks in the same expression. Extract negative guards into early returns or affirmative helper functions.

---

## 3. Subsystem Blast Radius & Impact Analysis

| File Path | Responsibility | Refactoring Pattern |
|---|---|---|
| `src/pages/instances/InstanceCardProjects.tsx` | Project liveness checking for instance cards | Split `!inst.is_running \|\| !proj.is_running` into discrete guard clauses; decompose `.some(...)` predicate |
| `src/pages/instances/instanceCardData.ts` | Instance card active task detection (`hasActiveTask`) | Decompose complex 4-operand chained expression into discrete affirmative conditions |
| `src/pages/instances/__tests__/instanceRunningPrompts.test.ts` | Unit test suite for instance card liveness | Synchronize test mirrors with discrete guard decomposition |
