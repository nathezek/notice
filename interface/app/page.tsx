"use client";

import { useState, useEffect, useCallback } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import SearchBar from "@/components/search_bar/search_bar";
import SearchResults from "@/components/search_results/search_results";
import InstantAnswer from "@/components/instant_answer/instant_answer";
import SubmitUrl from "@/components/submit_url/submit_url";
import { api, type SearchResponse } from "@/lib/api";
import { useAuth } from "@/lib/auth";

export default function HomePage() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const { user } = useAuth();

    const queryFromUrl = searchParams.get("q") || "";
    const [response, setResponse] = useState<SearchResponse | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [searchTime, setSearchTime] = useState<number | null>(null);

    const performSearch = useCallback(async (query: string) => {
        setLoading(true);
        setError(null);

        const start = performance.now();

        try {
            const res = await api.search(query, { limit: 20 });
            setResponse(res);
            setSearchTime(Math.round(performance.now() - start));
        } catch (err) {
            setError(err instanceof Error ? err.message : "Search failed");
            setResponse(null);
        } finally {
            setLoading(false);
        }
    }, []);

    // Search when URL query changes
    useEffect(() => {
        if (queryFromUrl) {
            performSearch(queryFromUrl);
        }
    }, [queryFromUrl, performSearch]);

    const handleSearch = (query: string) => {
        router.push(`/?q=${encodeURIComponent(query)}`);
    };

    const hasSearched = queryFromUrl !== "";

    return (
        <div className="mx-auto max-w-3xl px-4">
            {/* Hero / centered layout when no search */}
            {!hasSearched ? (
                <div className="flex min-h-[70vh] flex-col items-center justify-center">
                    <h1 className="mb-2 text-5xl font-bold tracking-tight text-neutral-700">
                        Notice
                    </h1>
                    <p className="mb-8 text-lg">
                        Search smarter. Personalized for you.
                    </p>

                    <div className="w-full max-w-xl">
                        <SearchBar onSearch={handleSearch} autoFocus />
                    </div>

                    {user && (
                        <p className="mt-4 text-xs">
                            Signed in as <span> {user.username}</span>
                            {" — "}your searches build your personal knowledge
                            graph
                        </p>
                    )}

                    <div className="mt-12 w-full max-w-xl">
                        <SubmitUrl />
                    </div>
                </div>
            ) : (
                /* Search results layout */
                <div className="py-6">
                    <div className="mb-6">
                        <SearchBar
                            initialQuery={queryFromUrl}
                            onSearch={handleSearch}
                            loading={loading}
                        />

                        {user && (
                            <p className="mt-2 text-xs">
                                Personalized for <span>{user.username}</span>
                            </p>
                        )}
                    </div>

                    {error && (
                        <div className="mb-4 rounded-lg p-3 text-sm">
                            {error}
                        </div>
                    )}

                    {response && (
                        <>
                            {response.instant_answer && (
                                <InstantAnswer
                                    answer={response.instant_answer}
                                />
                            )}

                            {searchTime !== null &&
                                response.results.length > 0 && (
                                    <p className="mb-1 text-xs">
                                        {searchTime}ms
                                    </p>
                                )}

                            <SearchResults
                                results={response.results}
                                total={response.total}
                                query={response.query}
                            />
                        </>
                    )}

                    <div className="mt-8">
                        <SubmitUrl />
                    </div>
                </div>
            )}
        </div>
    );
}
