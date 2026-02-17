import { create } from "zustand";

// --- Types ---
export interface WhoResult {
    type: "who";
    name: string;
    lifespan: string;
    known_for: string;
    achievements: string[];
}

export interface HowResult {
    type: "how";
    title: string;
    difficulty: string;
    steps: { step: number; instruction: string }[];
}

export interface WhatResult {
    type: "what";
    concept: string;
    definition: string;
    application: string;
    origin: string;
}

export interface WhenResult {
    type: "when";
    event: string;
    date: string;
    significance: string;
    timeline: string[];
}

export interface WhereResult {
    type: "where";
    location: string;
    region: string;
    facts: string[];
    climate: string;
}

export type SearchResult =
    | WhoResult
    | HowResult
    | WhatResult
    | WhenResult
    | WhereResult;

interface SearchState {
    inputQuery: string;
    result: SearchResult | null;
    isLoading: boolean;
    setInputQuery: (query: string) => void;
    setResult: (result: SearchResult | null) => void;
    setLoading: (loading: boolean) => void;
    resetSearch: () => void;
}

export const useSearchStore = create<SearchState>((set) => ({
    inputQuery: "",
    result: null,
    isLoading: false,
    setInputQuery: (query) => set({ inputQuery: query }),
    setResult: (result) => set({ result }),
    setLoading: (loading) => set({ isLoading: loading }),
    resetSearch: () => set({ result: null, inputQuery: "" }),
}));
