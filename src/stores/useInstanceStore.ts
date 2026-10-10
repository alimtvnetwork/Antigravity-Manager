/**
 * useInstanceStore — facade
 * 实际实现拆分至 ./useInstanceStore/ slices，此处组合以保持导入路径不变。
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
