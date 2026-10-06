import { describe, it, expect } from 'vitest';
import { resolveFocusTarget } from '../resolve-focus-target';

describe('resolveFocusTarget', () => {
  it('returns selectedInstanceAccountId when both are provided', () => {
    expect(resolveFocusTarget('acc-inst-1', 'acc-curr-2')).toBe('acc-inst-1');
  });

  it('falls back to currentAccountId when selectedInstanceAccountId is null or empty', () => {
    expect(resolveFocusTarget(null, 'acc-curr-2')).toBe('acc-curr-2');
    expect(resolveFocusTarget(undefined, 'acc-curr-2')).toBe('acc-curr-2');
    expect(resolveFocusTarget('   ', 'acc-curr-2')).toBe('acc-curr-2');
  });

  it('returns null when neither is provided', () => {
    expect(resolveFocusTarget(null, null)).toBeNull();
    expect(resolveFocusTarget(undefined, undefined)).toBeNull();
    expect(resolveFocusTarget('', '')).toBeNull();
    expect(resolveFocusTarget('   ', '   ')).toBeNull();
  });

  it('trims whitespace around valid IDs', () => {
    expect(resolveFocusTarget('  acc-inst-1  ', null)).toBe('acc-inst-1');
    expect(resolveFocusTarget(null, '  acc-curr-2  ')).toBe('acc-curr-2');
  });
});
