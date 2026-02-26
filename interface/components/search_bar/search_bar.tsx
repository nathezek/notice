"use client";

import { useState, useRef, useEffect } from "react";

interface SearchBarProps {
    initialQuery?: string;
    onSearch: (query: string) => void;
    loading?: boolean;
    autoFocus?: boolean;
}

export default function SearchBar({
    initialQuery = "",
    onSearch,
    loading = false,
    autoFocus = false,
}: SearchBarProps) {
    const [query, setQuery] = useState(initialQuery);
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (autoFocus && inputRef.current) {
            inputRef.current.focus();
        }
    }, [autoFocus]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        const trimmed = query.trim();
        if (trimmed) {
            onSearch(trimmed);
        }
    };

    return (
        <form onSubmit={handleSubmit} className="w-full">
            <div className="relative flex items-center">
                {/* Search icon */}
                <svg
                    className="absolute left-4 h-5 w-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    strokeWidth={2}
                >
                    <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    />
                </svg>

                <input
                    ref={inputRef}
                    type="text"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder="Search anything..."
                    className="w-full rounded-xl border py-3.5 pr-24 pl-12 text-base transition-colors"
                />

                <button
                    type="submit"
                    disabled={loading || !query.trim()}
                    className="absolute right-2 rounded-lg px-4 py-2 text-sm font-medium text-white transition-colors disabled:cursor-not-allowed disabled:opacity-40"
                >
                    {loading ? (
                        <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
                    ) : (
                        "Search"
                    )}
                </button>
            </div>
        </form>
    );
}
