import { create } from "zustand";

// --- Types ---

// Old types (keeping for backward compat if needed, or we can just nuke them if we are fully migrating)
// User wanted flexible "Universal" schema.

export interface UniversalResult {
    type: "universal"; // We'll tag it 'universal' for now
    summary: string;
    facts?: { label: string; value: string }[];
    related_topics?: string[];
    widgets?: { type: string; query: string }[];
}

// Fallback for old types if backend sends them (optional)
export interface OldResult {
    type: "who" | "what" | "how" | "when" | "where";
    [key: string]: any;
}

export interface ErrorResult {
    error: string;
}

export type SearchResult = UniversalResult | OldResult | ErrorResult;

interface SearchState {
    inputQuery: string;
    result: SearchResult | null;
    isLoading: boolean;
    hasSearched: boolean;
    setInputQuery: (query: string) => void;
    setResult: (result: SearchResult | null) => void;
    setLoading: (loading: boolean) => void;
    setHasSearched: (val: boolean) => void;
    resetSearch: () => void;
}

export const useSearchStore = create<SearchState>((set) => ({
    inputQuery: "",
    result: null,
    isLoading: false,
    hasSearched: false,
    setInputQuery: (query) => set({ inputQuery: query }),
    setResult: (result) => set({ result }),
    setLoading: (loading) => set({ isLoading: loading }),
    setHasSearched: (val) => set({ hasSearched: val }),
    resetSearch: () =>
        set({
            inputQuery: "",
            result: null,
            isLoading: false,
            hasSearched: false,
        }),
}));
