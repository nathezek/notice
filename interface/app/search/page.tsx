"use client";

import { useEffect, useCallback, Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import SearchResults from "@/components/search_results/search_results";
import InstantAnswer from "@/components/instant_answer/instant_answer";
import { api, type SearchResult } from "@/lib/api";
import { useSearchStore, type UniversalResult } from "@/stores/search_store";
import { useAuth } from "@/lib/auth";
import { SearchResultSkeleton } from "@/components/ui/skeleton";
import {
    WebsiteModal,
    type WebsiteMetadata,
} from "@/components/website/website_modal";
import Summary from "@/components/search_results/summary";

function SearchContent() {
    const searchParams = useSearchParams();
    const { user } = useAuth();

    const [selectedWebsite, setSelectedWebsite] =
        useState<WebsiteMetadata | null>(null);

    const {
        setInputQuery,
        setResult,
        setResultType,
        setLoading,
        setSummaryLoading,
        isLoading,
        result,
        isModalOpen,
        setModalOpen,
    } = useSearchStore();

    const queryFromUrl =
        searchParams.get("query") || searchParams.get("q") || "";

    const performSearch = useCallback(
        async (query: string) => {
            setLoading(true);
            setResult(null);

            try {
                const res = await api.search(query, { limit: 20 });

                // If it's an instant answer, we handle it separately or map it
                if (res.instant_answer) {
                    // ... handle specialized result types (math, etc)
                }

                setResult({
                    type: "universal",
                    title: query,
                    summary: res.ai_answer || "",
                    websites: res.results.map((r) => ({
                        url: r.url,
                        title: r.title || r.url,
                        snippet: r.snippet,
                    })),
                } as UniversalResult);

                setResultType("concept");
            } catch (err) {
                console.error("Search failed", err);
            } finally {
                setLoading(false);
            }
        },
        [setLoading, setResult, setResultType],
    );

    // Search when URL query changes or on mount
    useEffect(() => {
        if (queryFromUrl) {
            setInputQuery(queryFromUrl);
            performSearch(queryFromUrl);
        }
    }, [queryFromUrl, setInputQuery, performSearch]);

    return (
        <div className="mx-auto max-w-[90%] px-4 pt-32 pb-20">
            {isLoading ? (
                <SearchResultSkeleton />
            ) : (
                <div className="grid grid-cols-2 gap-x-12">
                    {/* Instant Answer (Calculation, etc.) would go here */}


                    {/* Web Results */}
                    {result?.type === "universal" && result.websites && (
                        <SearchResults
                            results={result.websites.map((w, i) => ({
                                id: i.toString(),
                                url: w.url,
                                title: w.title,
                                snippet: w.snippet || "",
                                score: null,
                            }))}
                            total={result.websites.length}
                            query={queryFromUrl}
                            onResultClick={(r) => {
                                setSelectedWebsite({
                                    url: r.url,
                                    title: r.title || r.url,
                                });
                                setModalOpen(true);
                            }}
                        />
                    )}

                    {/* AI Answer Block */}
                    {result?.type === "universal" && result.summary && (
                        <div className="mb-12 pr-12">
                            <Summary answer={result.summary} />
                        </div>
                    )}



                </div>
            )}

            <WebsiteModal
                isOpen={isModalOpen}
                onClose={() => setModalOpen(false)}
                website={selectedWebsite}
            />
        </div>
    );
}

export default function SearchPage() {
    return (
        <Suspense
            fallback={
                <div className="flex min-h-screen items-center justify-center">
                    <SearchResultSkeleton />
                </div>
            }
        >
            <SearchContent />
        </Suspense>
    );
}
