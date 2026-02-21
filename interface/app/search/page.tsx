"use client";

import { useSearchParams } from "next/navigation";
import { useEffect, useState, useRef } from "react";
import {
    useSearchStore,
    ResultType,
    UniversalResult,
    MathResult,
    UnitResult,
    CurrencyResult,
    TimerResult,
    Website,
} from "@/stores/search_store";
import { UniversalBlock } from "@/components/blocks/universal_block";
import { CalculatorBlock } from "@/components/blocks/calculator_block";
import { ConverterBlock } from "@/components/blocks/converter_block";
import { TimerBlock } from "@/components/blocks/timer_block";

import { MOCK_WEBSITES, WebsiteData } from "@/mock_data/website_mock_data";
import { WebsiteList } from "@/components/website/website_list";
import { WebsiteModal } from "@/components/website/website_modal";
import { IndexerComponent } from "@/components/indexer/indexer";

import { Suspense } from "react";
import {
    SearchResultSkeleton,
    WebsiteListSkeleton,
} from "@/components/ui/skeleton";
import { motion } from "motion/react";

function SearchResults() {
    const searchParams = useSearchParams();
    const query = searchParams.get("query");

    const [selectedWebsite, setSelectedWebsite] = useState<WebsiteData | null>(
        null,
    );

    const {
        result,
        resultType,
        correctedQuery,
        isLoading,
        isSummaryLoading,
        setLoading,
        setSummaryLoading,
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
                // Step A: Fast Web Search (/search/web)
                const response = await fetch(
                    "http://localhost:4000/search/web",
                    {
                        method: "POST",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ query }),
                    },
                );
                const data = await response.json();
                console.log("Web Search Response:", data);

                const serverResultType = data.result_type as ResultType;
                setResultType(serverResultType);
                setCorrectedQuery(data.corrected_query ?? null);

                let parsedResult;
                try {
                    parsedResult = JSON.parse(data.content);
                } catch (e) {
                    console.error("JSON Parse Error:", e);
                    parsedResult = { error: "Failed to parse response" };
                    setResultType("error");
                    setCorrectedQuery(null);
                }
                setResult(parsedResult);
                setLoading(false);

                // Step B: Background AI Summary (/search/summary)
                // Only for "concept" results where we need an AI summary
                if (serverResultType === "concept") {
                    setSummaryLoading(true);
                    try {
                        const summaryResponse = await fetch(
                            "http://localhost:4000/search/summary",
                            {
                                method: "POST",
                                headers: { "Content-Type": "application/json" },
                                body: JSON.stringify({
                                    query,
                                    urls:
                                        parsedResult.websites?.map(
                                            (w: Website) => w.url,
                                        ) || [],
                                }),
                            },
                        );
                        const summaryData = await summaryResponse.json();
                        console.log("Summary Response:", summaryData);

                        const freshResult = JSON.parse(summaryData.content);
                        // Merge with initial parsedResult to ensure websites are preserved
                        // if Gemini fails to return them or returns an error.
                        setResult({
                            ...(parsedResult as UniversalResult),
                            ...freshResult,
                        });
                    } catch (summaryError) {
                        console.error("Summary fetch failed:", summaryError);
                    } finally {
                        setSummaryLoading(false);
                    }
                }
            } catch (error) {
                console.error("Search failed:", error);
                setResultType("error");
                setCorrectedQuery(null);
                setLoading(false);
            }
        };

        fetchResults();
    }, [
        query,
        setInputQuery,
        setHasSearched,
        setLoading,
        setSummaryLoading,
        setResult,
        setResultType,
        setCorrectedQuery,
    ]);

    const renderResult = () => {
        if (!result || isLoading) return null;

        if ("error" in result) {
            return (
                <div className="serif-font flex flex-col items-center justify-center">
                    {/*<span className="my-8 text-center font-sans text-9xl text-neutral-700 dark:text-neutral-600">
                        \(^ _ ^)/
                    </span>*/}

                    <h3 className="text-normal mt-12 w-96 text-center text-lg leading-relaxed tracking-[0.015rem] opacity-80">
                        It appears that we a currently unable to process your
                        request, would you mind refreshing the page? <br />
                        <br />
                        <br />
                        <span className="text-center text-sm">
                            {result.error}
                        </span>
                    </h3>
                </div>
            );
        }

        switch (resultType) {
            case "math":
                return <CalculatorBlock data={result as MathResult} />;
            case "unit_conversion":
                return (
                    <ConverterBlock
                        type="unit_conversion"
                        data={result as UnitResult}
                    />
                );
            case "currency_conversion":
                return (
                    <ConverterBlock
                        type="currency_conversion"
                        data={result as CurrencyResult}
                    />
                );
            case "timer":
                return <TimerBlock data={result as TimerResult} />;
            case "concept":
            default:
                return (
                    <div className="relative">
                        <UniversalBlock data={result as UniversalResult} />
                        {isSummaryLoading && (
                            <div className="absolute inset-0 flex items-center justify-center bg-white/50 backdrop-blur-sm transition-opacity duration-300 dark:bg-neutral-900/50">
                                <div className="flex flex-col items-center gap-4">
                                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-neutral-300 border-t-neutral-800 dark:border-neutral-700 dark:border-t-neutral-200"></div>
                                    <p className="animate-pulse text-sm font-medium">
                                        Summarizing web results...
                                    </p>
                                </div>
                            </div>
                        )}
                    </div>
                );
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
                <div
                    className={
                        showSidebar ? "lg:col-span-2" : "w-full max-w-2xl"
                    }
                >
                    {/* "Did you mean..." banner */}
                    {correctedQuery && !isLoading && (
                        <div className="mb-4 text-sm text-neutral-500 dark:text-neutral-400">
                            Showing results for{" "}
                            <span className="font-medium text-neutral-900 dark:text-neutral-100">
                                {correctedQuery}
                            </span>
                            {" — Did you mean "}
                            <button
                                className="underline underline-offset-2 hover:text-neutral-900 dark:hover:text-neutral-100"
                                onClick={() =>
                                    (window.location.href = `/search?query=${encodeURIComponent(query ?? "")}`)
                                }
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
                                <div className="flex flex-col gap-8">
                                    <IndexerComponent />
                                    <WebsiteList
                                        websites={
                                            result &&
                                            "websites" in result &&
                                            Array.isArray(result.websites) &&
                                            result.websites.length > 0
                                                ? (
                                                      result as UniversalResult
                                                  ).websites!.map(
                                                      (w, idx: number) => ({
                                                          id: `site-${idx}`,
                                                          url: w.url,
                                                          title: w.title,
                                                          snippet:
                                                              w.snippet ||
                                                              new URL(w.url)
                                                                  .hostname,
                                                          imageUrl:
                                                              w.imageUrl ||
                                                              `https://image.thum.io/get/width/400/crop/800/noanimate/${w.url}`,
                                                      }),
                                                  )
                                                : MOCK_WEBSITES
                                        }
                                        onWebsiteClick={setSelectedWebsite}
                                    />
                                </div>
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
