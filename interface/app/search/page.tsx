"use client";

import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { useSearchStore, ResultType } from "@/stores/search_store";
import { UniversalBlock } from "@/components/blocks/universal_block";
import { CalculatorBlock } from "@/components/blocks/calculator_block";
import { ConverterBlock } from "@/components/blocks/converter_block";

import { MOCK_WEBSITES, WebsiteData } from "@/mock_data/website_mock_data";
import { WebsiteList } from "@/components/website/website_list";
import { WebsiteModal } from "@/components/website/website_modal";

import { Suspense } from "react";
import { SearchResultSkeleton, WebsiteListSkeleton } from "@/components/ui/skeleton";
import { motion } from "motion/react";

function SearchResults() {
    const searchParams = useSearchParams();
    const query = searchParams.get("query");

    const [selectedWebsite, setSelectedWebsite] = useState<WebsiteData | null>(null);

    const {
        result,
        resultType,
        isLoading,
        setLoading,
        setResult,
        setResultType,
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

                // Store the result_type so we know which block to render
                const serverResultType = data.result_type as ResultType;
                setResultType(serverResultType);

                let parsedResult;
                try {
                    parsedResult = JSON.parse(data.content);
                    console.log("Parsed Result:", parsedResult);
                } catch (e) {
                    console.error("JSON Parse Error:", e);
                    parsedResult = { error: "Failed to parse response" };
                    setResultType("error");
                }

                setResult(parsedResult);
            } catch (error) {
                console.error("Search failed:", error);
                setResultType("error");
            } finally {
                setLoading(false);
            }
        };

        fetchResults();
    }, [query, setInputQuery, setHasSearched, setLoading, setResult, setResultType]);

    const renderResult = () => {
        if (!result || isLoading) return null;

        if ("error" in result) {
            return (
                <div className="rounded-xl border border-red-200 bg-red-50 p-6 text-red-700 dark:border-red-900/30 dark:bg-red-900/10 dark:text-red-400">
                    <h3 className="mb-2 font-semibold">Search Error</h3>
                    <p>{result.error}</p>
                </div>
            );
        }

        switch (resultType) {
            case "math":
                return <CalculatorBlock data={result as any} />;
            case "unit_conversion":
                return <ConverterBlock type="unit_conversion" data={result as any} />;
            case "currency_conversion":
                return <ConverterBlock type="currency_conversion" data={result as any} />;
            case "concept":
            default:
                return <UniversalBlock data={result as any} />;
        }
    };

    // Hide sidebar for structured (instant) result types
    const showSidebar = resultType === "concept" || resultType === null;

    return (
        <div className="flex min-h-screen flex-col items-center p-8 font-sans md:p-12 dark:bg-neutral-900 dark:text-white">
            <WebsiteModal
                isOpen={!!selectedWebsite}
                onClose={() => setSelectedWebsite(null)}
                website={selectedWebsite}
            />

            <motion.div
                className={`mt-20 grid w-full max-w-6xl grid-cols-1 gap-12 ${showSidebar ? "lg:grid-cols-4 lg:pl-20" : ""}`}
                initial={{ opacity: 0, y: 24 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.55, duration: 0.5, ease: "easeOut" }}
            >
                {/* Left Column: Result */}
                <div className={showSidebar ? "lg:col-span-3" : "w-full max-w-2xl"}>
                    {isLoading && <SearchResultSkeleton />}
                    {renderResult()}
                    {!result && !isLoading && !query && (
                        <div className="mt-12 text-center text-neutral-400">
                            Enter a query to start searching.
                        </div>
                    )}
                </div>

                {/* Right Column: Website List — only for concept results */}
                {showSidebar && (
                    <div className="hidden lg:col-span-1 lg:block">
                        <div className="sticky top-24">
                            {isLoading ? (
                                <WebsiteListSkeleton />
                            ) : (
                                <WebsiteList
                                    websites={MOCK_WEBSITES}
                                    onWebsiteClick={setSelectedWebsite}
                                />
                            )}
                        </div>
                    </div>
                )}
            </motion.div>
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
