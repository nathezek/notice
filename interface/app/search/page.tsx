"use client";

import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { useSearchStore } from "@/stores/search_store";
import { WhoBlock } from "@/components/blocks/concept_query_blocks/who_block";
import { WhatBlock } from "@/components/blocks/concept_query_blocks/what_block";
import { HowBlock } from "@/components/blocks/concept_query_blocks/how_block";
import { WhenBlock } from "@/components/blocks/concept_query_blocks/when_block";
import { WhereBlock } from "@/components/blocks/concept_query_blocks/where_block";

import { UniversalBlock } from "@/components/blocks/universal_block";

import { MOCK_WEBSITES, WebsiteData } from "@/mock_data/website_mock_data";
import { WebsiteList } from "@/components/website/website_list";
import { WebsiteModal } from "@/components/website/website_modal";

import { Suspense } from "react";
import { SearchResultSkeleton } from "@/components/ui/skeleton";

function SearchResults() {
    const searchParams = useSearchParams();
    const query = searchParams.get("query");

    // Modal state
    const [selectedWebsite, setSelectedWebsite] = useState<WebsiteData | null>(null);

    // global store state
    const {
        result,
        isLoading,
        setLoading,
        setResult,
        setHasSearched,
        setInputQuery
    } = useSearchStore();

    useEffect(() => {
        if (!query) return;

        // Update input query in store so Navbar reflects URL
        setInputQuery(query);
        setHasSearched(true);

        const fetchResults = async () => {
            setLoading(true);
            try {
                const response = await fetch("http://localhost:4000/search", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ query }),
                });
                const data = await response.json();
                console.log("Raw Server Response:", data);

                let parsedResult;
                try {
                    parsedResult = JSON.parse(data.content);
                    console.log("Parsed Result:", parsedResult);
                } catch (e) {
                    console.error("JSON Parse Error:", e);
                    parsedResult = { error: "Failed to parse response" };
                }

                setResult(parsedResult);
            } catch (error) {
                console.error("Search failed:", error);
            } finally {
                setLoading(false);
            }
        };

        fetchResults();
    }, [query, setInputQuery, setHasSearched, setLoading, setResult]);

    return (
        <div className="flex min-h-screen flex-col items-center p-8 font-sans dark:bg-neutral-900 dark:text-white md:p-12">

            {/* Modal */}
            <WebsiteModal
                isOpen={!!selectedWebsite}
                onClose={() => setSelectedWebsite(null)}
                website={selectedWebsite}
            />

            {/* --- Main Content Grid --- */}
            <div className="mt-20 grid w-full max-w-6xl grid-cols-1 gap-12 lg:grid-cols-3">

                {/* --- Left Column (2/3): Search Results --- */}
                <div className="lg:col-span-2">
                    {isLoading && (
                        <SearchResultSkeleton />
                    )}

                    {result && !isLoading && (
                        <div className="space-y-6">
                            {/* --- Error Handling --- */}
                            {/* @ts-ignore */}
                            {result.error && (
                                <div className="rounded-xl border border-red-200 bg-red-50 p-6 text-red-700 dark:border-red-900/30 dark:bg-red-900/10 dark:text-red-400">
                                    <h3 className="mb-2 font-semibold">Search Error</h3>
                                    <p>{result.error}</p>
                                </div>
                            )}

                            {/* --- NEW: Universal Block --- */}
                            {/* 
                                Check for 'summary' key to detect universal result, 
                                as we are transitioning schemas. 
                             */}
                            {/* @ts-ignore - straightforward check for new schema */}
                            {result.summary && (
                                // @ts-ignore
                                <UniversalBlock data={result} />
                            )}

                            {/* --- OLD BLOCKS (Fallback) --- */}
                            {result.type === "what" && <WhatBlock data={result} />}
                            {result.type === "who" && <WhoBlock data={result} />}
                            {result.type === "how" && <HowBlock data={result} />}
                            {result.type === "when" && <WhenBlock data={result} />}
                            {result.type === "where" && (
                                <WhereBlock data={result} />
                            )}
                        </div>
                    )}
                    {!result && !isLoading && !query && (
                        <div className="mt-12 text-center text-neutral-400">
                            Enter a query to start searching.
                        </div>
                    )}
                </div>

                {/* --- Right Column (1/3): Website List (Mock Data) --- */}
                <div className="hidden lg:block lg:col-span-1">
                    <div className="sticky top-24">
                        <WebsiteList
                            websites={MOCK_WEBSITES}
                            onWebsiteClick={setSelectedWebsite}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
}

export default function SearchPage() {
    return (
        <Suspense fallback={<div className="flex h-screen w-full items-center justify-center">Loading...</div>}>
            <SearchResults />
        </Suspense>
    );
}
