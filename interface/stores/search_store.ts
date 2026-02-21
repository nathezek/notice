import { create } from "zustand";

export interface Website {
    url: string;
    title: string;
    snippet?: string;
    imageUrl?: string;
}

export interface UniversalResult {
    type: "universal";
    title: string;
    summary: string;
    facts?: { label: string; value: string }[];
    related_topics?: string[];
    widgets?: { type: string; query: string }[];
    websites?: Website[];
}

export interface MathResult {
    expression: string;
    result: string;
}

export interface UnitResult {
    amount: number;
    from: string;
    to: string;
    result: string;
    category: string;
}

export interface CurrencyResult {
    amount: number;
    from: string;
    to: string;
    result: string;
    rate: string;
}

export interface ErrorResult {
    error: string;
}

export interface TimerResult {
    seconds: number;
    query: string;
}

export type SearchResult = UniversalResult | MathResult | UnitResult | CurrencyResult | ErrorResult | TimerResult;

// The result_type string from the server, used to pick the right block
export type ResultType = "concept" | "math" | "unit_conversion" | "currency_conversion" | "error" | "timer";

interface SearchState {
    inputQuery: string;
    result: SearchResult | null;
    resultType: ResultType | null;
    correctedQuery: string | null;
    isLoading: boolean;
    isSummaryLoading: boolean;
    hasSearched: boolean;
    setInputQuery: (query: string) => void;
    setResult: (result: SearchResult | null) => void;
    setResultType: (type: ResultType | null) => void;
    setCorrectedQuery: (q: string | null) => void;
    setLoading: (loading: boolean) => void;
    setSummaryLoading: (loading: boolean) => void;
    setHasSearched: (val: boolean) => void;
    resetSearch: () => void;
}

export const useSearchStore = create<SearchState>((set) => ({
    inputQuery: "",
    result: null,
    resultType: null,
    correctedQuery: null,
    isLoading: false,
    isSummaryLoading: false,
    hasSearched: false,
    setInputQuery: (query) => set({ inputQuery: query }),
    setResult: (result) => set({ result }),
    setResultType: (type) => set({ resultType: type }),
    setCorrectedQuery: (q) => set({ correctedQuery: q }),
    setLoading: (loading) => set({ isLoading: loading }),
    setSummaryLoading: (loading) => set({ isSummaryLoading: loading }),
    setHasSearched: (val) => set({ hasSearched: val }),
    resetSearch: () =>
        set({
            inputQuery: "",
            result: null,
            resultType: null,
            correctedQuery: null,
            isLoading: false,
            isSummaryLoading: false,
            hasSearched: false,
        }),
}));
