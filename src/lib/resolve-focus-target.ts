/**
 * Resolves the target account ID to focus on.
 * Prioritizes the account bound to the currently selected instance,
 * falling back to the global current account ID, or null if neither exists.
 */
export function resolveFocusTarget(
  selectedInstanceAccountId: string | null | undefined,
  currentAccountId: string | null | undefined,
): string | null {
  if (selectedInstanceAccountId && selectedInstanceAccountId.trim().length > 0) {
    return selectedInstanceAccountId.trim();
  }
  if (currentAccountId && currentAccountId.trim().length > 0) {
    return currentAccountId.trim();
  }
  return null;
}
