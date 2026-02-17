"use client";
import { QuerySearchForm } from "@/modules/forms/query_search_form";
import { ThemeSwitcher } from "@/theme/theme_switcher";
import { WhoBlock } from "@/components/blocks/concept_query_blocks/who_block";
import { WhatBlock } from "@/components/blocks/concept_query_blocks/what_block";
import { HowBlock } from "@/components/blocks/concept_query_blocks/how_block";
import { WhenBlock } from "@/components/blocks/concept_query_blocks/when_block";
import { WhereBlock } from "@/components/blocks/concept_query_blocks/where_block";
import { useSearchStore } from "@/stores/search_store";

export default function Home() {
    const {
        inputQuery,
        setInputQuery,
        result,
        setResult,
        isLoading,
        setLoading,
    } = useSearchStore();

    const handleSearch = async (e?: React.FormEvent) => {
        if (e) e.preventDefault();
        if (!inputQuery.trim()) return;

        setLoading(true);
        setResult(null);

        try {
            const response = await fetch("http://localhost:4000/search", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ query: inputQuery }),
            });

            const data = await response.json();
            // Parse the JSON string from Rust
            setResult(JSON.parse(data.content));
        } catch (error) {
            console.error("Search failed:", error);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="flex min-h-screen flex-col items-center p-12 font-sans dark:bg-neutral-900 dark:text-white">
            <h1 className="mb-8 text-3xl font-bold">Smart Search</h1>
            <ThemeSwitcher />

            {/* --- Search Bar --- */}
            <QuerySearchForm
                handleSearch={handleSearch}
                inputQuery={inputQuery}
                setInputQuery={setInputQuery}
            />

            {/* --- Results Area --- */}
            <div className="mt-12 w-full max-w-2xl">
                {isLoading && (
                    <div className="animate-pulse text-center opacity-60">
                        fetching request...
                    </div>
                )}

                {result && (
                    <div className="space-y-6">
                        {/* --- BLOCK: WHAT (Concept) --- */}
                        {result.type === "what" && <WhatBlock data={result} />}
                        {/* --- BLOCK: WHO (Person) --- */}
                        {result.type === "who" && <WhoBlock data={result} />}
                        {/* --- BLOCK: HOW (Process) --- */}
                        {result.type === "how" && <HowBlock data={result} />}
                        {/* --- BLOCK: WHEN (Event) --- */}
                        {result.type === "when" && <WhenBlock data={result} />}
                        {/* --- BLOCK: WHERE (Location) --- */}
                        {result.type === "where" && (
                            <WhereBlock data={result} />
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
