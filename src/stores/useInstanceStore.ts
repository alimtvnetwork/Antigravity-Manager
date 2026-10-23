/**
 * useInstanceStore — facade
 * The actual implementation is split into ./useInstanceStore/ slices and combined here to keep the import path unchanged.
 */
import { create } from 'zustand';
import type { InstanceState } from './useInstanceStore/types';
import { createCrudSlice } from './useInstanceStore/crud';
import { createLifecycleSlice } from './useInstanceStore/lifecycle';
import { createSwitcherSlice } from './useInstanceStore/switcher';
import { createSmartSlice } from './useInstanceStore/smart';

export type { InstanceState } from './useInstanceStore/types';

export const useInstanceStore = create<InstanceState>()((set, get) => ({
    instances: [],
    activeInstanceId: 'default',
    switcherStatus: null,
    isLoading: false,
    error: null,

    ...createCrudSlice(set, get),
    ...createLifecycleSlice(set, get),
    ...createSwitcherSlice(set, get),
    ...createSmartSlice(set, get),
}));
