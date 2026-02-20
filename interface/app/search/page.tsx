"use client";

import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { useSearchStore } from "@/stores/search_store";
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
    const [selectedWebsite, setSelectedWebsite] = useState<WebsiteData | null>(
        null,
    );

    const {
        result,
        isLoading,
        setLoading,
        setResult,
        setHasSearched,
        setInputQuery,
    } = useSearchStore();

    useEffect(() => {
        if (!query) return;

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
        <div className="flex min-h-screen flex-col items-center p-8 font-sans md:p-12 dark:bg-neutral-900 dark:text-white">
            {/* Modal */}
            <WebsiteModal
                isOpen={!!selectedWebsite}
                onClose={() => setSelectedWebsite(null)}
                website={selectedWebsite}
            />

            {/* --- Main Content Grid --- */}
            <div className="mt-20 grid w-full max-w-6xl grid-cols-1 gap-12 lg:grid-cols-4">
                {/* --- Left Column (2/3): Search Results --- */}
                <div className="lg:col-span-3">
                    {isLoading && <SearchResultSkeleton />}

                    {result && !isLoading && (
                        <div className="space-y-6">
                            {"error" in result ? (
                                <div className="rounded-xl border border-red-200 bg-red-50 p-6 text-red-700 dark:border-red-900/30 dark:bg-red-900/10 dark:text-red-400">
                                    <h3 className="mb-2 font-semibold">Search Error</h3>
                                    <p>{result.error}</p>
                                </div>
                            ) : (
                                <UniversalBlock data={result} />
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
                <div className="hidden lg:col-span-1 lg:block">
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
        <Suspense
            fallback={
                <div className="flex h-screen w-full items-center justify-center">
                    Loading...
                </div>
            }
        >
            <SearchResults />
        </Suspense>
    );
}
