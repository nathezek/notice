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
import { useRef } from "react";

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
        isSummaryLoading,
        setSummaryLoading,
        isLoading,
        result,
        isModalOpen,
        setModalOpen,
        discoveryStatus,
        setDiscoveryStatus,
    } = useSearchStore();

    const lastQueryRef = useRef<string | null>(null);
    const pollCountRef = useRef(0);

    const queryFromUrl =
        searchParams.get("query") || searchParams.get("q") || "";

    const performSearch = useCallback(
        async (query: string) => {
            if (query === lastQueryRef.current) return;
            lastQueryRef.current = query;
            pollCountRef.current = 0;

            setLoading(true);
            setResult(null);
            setDiscoveryStatus("idle");
            setSummaryLoading(false);

            try {
                // Step A: Fast Search (Web Results Only)
                const res = await api.search(query, { limit: 20 });

                setResult({
                    type: "universal",
                    title: query,
                    summary: "", // Decoupled
                    websites: res.results.map((r) => ({
                        url: r.url,
                        title: r.title || r.url,
                        snippet: r.snippet,
                    })),
                } as UniversalResult);

                setResultType("concept");
                setLoading(false); // Display websites immediately

                // Step B: Handle Discovery
                if (res.discovery_triggered && res.results.length === 0) {
                    setDiscoveryStatus("preparing");
                    startPolling(query);
                }

                // Step C: Fetch AI Summary in background
                if (res.results.length > 0) {
                    fetchSummary(query);
                }
            } catch (err) {
                console.error("Search failed", err);
                setLoading(false);
            }
        },
        [setLoading, setResult, setResultType, setDiscoveryStatus, setSummaryLoading],
    );

    const fetchSummary = async (query: string) => {
        setSummaryLoading(true);
        try {
            const summaryRes = await api.searchSummary(query);
            setResult({
                type: "universal",
                title: summaryRes.title,
                summary: summaryRes.summary,
                websites: useSearchStore.getState().result?.type === "universal"
                    ? (useSearchStore.getState().result as UniversalResult).websites
                    : []
            } as UniversalResult);
        } catch (err) {
            console.error("Summary fetch failed", err);
        } finally {
            setSummaryLoading(false);
        }
    };

    const startPolling = (query: string) => {
        const poll = async () => {
            if (pollCountRef.current >= 5 || lastQueryRef.current !== query) return;
            pollCountRef.current++;

            try {
                const res = await api.search(query, { limit: 20 });
                if (res.results.length > 0) {
                    setResult({
                        type: "universal",
                        title: query,
                        summary: "",
                        websites: res.results.map((r) => ({
                            url: r.url,
                            title: r.title || r.url,
                            snippet: r.snippet,
                        })),
                    } as UniversalResult);
                    setDiscoveryStatus("ready");
                    fetchSummary(query);
                    return;
                }
            } catch (e) {
                console.error("Polling failed", e);
            }

            setTimeout(poll, 3000);
        };

        setTimeout(poll, 3000);
    };

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
                    {result?.type === "universal" && (
                        <SearchResults
                            results={(result.websites || []).map((w, i) => ({
                                id: i.toString(),
                                url: w.url,
                                title: w.title,
                                snippet: w.snippet || "",
                                score: null,
                            }))}
                            total={result.websites?.length || 0}
                            query={queryFromUrl}
                            discoveryStatus={discoveryStatus}
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
                    {result?.type === "universal" && (result.summary || isSummaryLoading) && (
                        <div className="mb-12 pr-12">
                            <Summary
                                title={result.title || "Overview"}
                                answer={result.summary}
                                isLoading={isSummaryLoading}
                            />
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
