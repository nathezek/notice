"use client";

import { useSearchParams } from "next/navigation";
import { useEffect, useState, useRef } from "react";
import { useSearchStore, ResultType } from "@/stores/search_store";
import { UniversalBlock } from "@/components/blocks/universal_block";
import { CalculatorBlock } from "@/components/blocks/calculator_block";
import { ConverterBlock } from "@/components/blocks/converter_block";
import { TimerBlock } from "@/components/blocks/timer_block";

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
        correctedQuery,
        isLoading,
        setLoading,
        setResult,
        setResultType,
        setCorrectedQuery,
        setHasSearched,
        setInputQuery,
    } = useSearchStore();

    const lastQueryRef = useRef<string | null>(null);

    useEffect(() => {
        if (!query) return;
        if (query === lastQueryRef.current) return;
        lastQueryRef.current = query;

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

                // Store corrected query for banner
                setCorrectedQuery(data.corrected_query ?? null);
                let parsedResult;
                try {
                    parsedResult = JSON.parse(data.content);
                    console.log("Parsed Result:", parsedResult);
                } catch (e) {
                    console.error("JSON Parse Error:", e);
                    parsedResult = { error: "Failed to parse response" };
                    setResultType("error");
                    setCorrectedQuery(null);
                }

                setResult(parsedResult);
            } catch (error) {
                console.error("Search failed:", error);
                setResultType("error");
                setCorrectedQuery(null);
            } finally {
                setLoading(false);
            }
        };

        fetchResults();
    }, [query, setInputQuery, setHasSearched, setLoading, setResult, setResultType, setCorrectedQuery]);

    const renderResult = () => {
        if (!result || isLoading) return null;

        if ("error" in result) {
            return (
                <div className="serif-font flex flex-col items-center justify-center">
                    <span className="text-9xl font-sans my-8 text-center text-neutral-700 dark:text-neutral-200">\(^Д^)/</span>

                    <h3 className="mb-2 text-center text-normal tracking-[0.015rem] leading-relaxed w-96 text-lg opacity-80">It appears that we a currently unable to process your request, would you mind refreshing the page?</h3>
                    <p className="text-center text-sm">{result.error}</p>
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
            case "timer":
                return <TimerBlock data={result as any} />;
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
                className={`mt-20 grid w-full max-w-6xl grid-cols-1 gap-12 ${showSidebar ? "lg:grid-cols-3 lg:pl-20" : ""}`}
                initial={{ opacity: 0, y: 24 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.55, duration: 0.5, ease: "easeOut" }}
            >
                {/* Left Column: Result */}
                <div className={showSidebar ? "lg:col-span-2" : "w-full max-w-2xl"}>
                    {/* "Did you mean..." banner */}
                    {correctedQuery && !isLoading && (
                        <div className="mb-4 text-sm text-neutral-500 dark:text-neutral-400">
                            Showing results for{" "}
                            <span className="font-medium text-neutral-900 dark:text-neutral-100">{correctedQuery}</span>
                            {" — Did you mean "}
                            <button
                                className="underline underline-offset-2 hover:text-neutral-900 dark:hover:text-neutral-100"
                                onClick={() => window.location.href = `/search?query=${encodeURIComponent(query ?? "")}`}
                            >
                                {query}
                            </button>
                            {query?.endsWith("?") ? "" : "?"}
                        </div>
                    )}

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
                                    websites={
                                        result && "websites" in result && Array.isArray(result.websites) && result.websites.length > 0
                                            ? result.websites.map((w: any, idx: number) => ({
                                                id: `site-${idx}`,
                                                url: w.url,
                                                title: w.title,
                                                snippet: w.snippet || new URL(w.url).hostname,
                                                imageUrl: w.imageUrl || `https://image.thum.io/get/width/400/crop/800/noanimate/${w.url}`
                                            }))
                                            : MOCK_WEBSITES
                                    }
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
