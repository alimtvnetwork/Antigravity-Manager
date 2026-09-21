import { create } from 'zustand';

export interface WindowBounds {
    x: number;
    y: number;
    width: number;
    height: number;
}

interface ViewState {
    isMiniView: boolean;
    savedBounds: WindowBounds | null;
    setMiniView: (isMini: boolean) => void;
    setSavedBounds: (bounds: WindowBounds | null) => void;
    toggleMiniView: () => void;
}

export const useViewStore = create<ViewState>((set) => ({
    isMiniView: false,
    savedBounds: null,
    setMiniView: (isMini) => set({ isMiniView: isMini }),
    setSavedBounds: (bounds) => set({ savedBounds: bounds }),
    toggleMiniView: () => set((state) => ({ isMiniView: !state.isMiniView })),
}));
