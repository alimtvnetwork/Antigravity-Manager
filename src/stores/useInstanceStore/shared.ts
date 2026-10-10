// Shared mutable state for instance selection race protection.
// Object wrapper (instead of a bare `let`) so slices can mutate it through an import binding.
export const instanceSelectionEpoch = { value: 0 };
