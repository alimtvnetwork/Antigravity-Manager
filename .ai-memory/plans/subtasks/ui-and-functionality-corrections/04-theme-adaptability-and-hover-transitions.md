# Subtask 04: Theme Adaptability & CSS3 Smooth Non-Displacement Hover Effects

- **Slug**: `theme-adaptability-and-hover-transitions`
- **Component**: Prompt Tree Theme Token System & Component Transitions
- **Primary Files**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
  - `src/components/instances/PromptTreeView/ConversationNode.tsx`
- **Status**: QUEUED

---

## 1. Problem Statement
The user explicitly stated:
> "Also, can you improve the UI a little bit better, based on the themes? If I go into the One Dark Pro, the theme still actually has purple things. If I go into other colors, the middle portion is always purple. It does not change based on that. Hover and other effects should be changing based on the theme that we pick. It is not happening. The cards, it is actually changing in movement, like position. Just makes things bigger or a little bit slides of the positioning. Never change the positioning of the item on hover, change the coloring. With a CSS3 animation, like 0.3 or 0.5 seconds so that it feels very nice. You can play a contrast role, like the background can be a little bit lighter, the text color can be darker. This could be one way. I'm not saying that you have to apply and test all the time, but this is just an idea."

## 2. Technical Requirements
1. **Purge Hardcoded Purple Values**:
   - Replace hardcoded `#120a1f`, `#40285e`, `border-purple-*`, `bg-purple-*` across PromptTreeView with theme-adaptive CSS classes or CSS variables that adapt to active themes (including One Dark Pro).
   - Ensure the middle detail pane reflects the chosen theme background and borders rather than persistent purple.
2. **Zero-Displacement Hover Effects**:
   - Audit all card, node, and button styles in `PromptTreeView`.
   - Remove layout-shifting transforms: `hover:scale-*`, `hover:-translate-y-*`, margin/padding jumps on hover.
   - Replace with smooth CSS3 transitions: `transition-colors duration-300 ease-in-out` or `transition-all duration-300 ease-in-out` modifying solely `background-color`, `border-color`, `box-shadow`, and `color`.
3. **Contrast Tuning**:
   - On hover, gently lift the surface background brightness and enhance border contrast for a clean, modern aesthetic.

## 3. Verification Criteria
- Switching themes (e.g. One Dark Pro) adapts the middle portion without stuck purple elements.
- Hovering over cards/items smoothly alters colors without moving or displacing items.
