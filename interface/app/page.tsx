"use client";
import { QuerySearchForm } from "@/modules/forms/query_search_form";
import { ThemeSwitcher } from "@/theme/theme_switcher";
import { IconSearch, IconX } from "@tabler/icons-react";
import { useState } from "react";

// --- 1. Define the Shapes (The Menu) ---

interface WhoResult {
    type: "who";
    name: string;
    lifespan: string;
    known_for: string;
    achievements: string[];
}

interface HowResult {
    type: "how";
    title: string;
    difficulty: string;
    steps: { step: number; instruction: string }[];
}

interface WhatResult {
    type: "what";
    concept: string;
    definition: string;
    application: string;
    origin: string;
}

interface WhenResult {
    type: "when";
    event: string;
    date: string;
    significance: string;
    timeline: string[];
}

interface WhereResult {
    type: "where";
    location: string;
    region: string;
    facts: string[];
    climate: string;
}

type SearchResult =
    | WhoResult
    | HowResult
    | WhatResult
    | WhenResult
    | WhereResult;

export default function Home() {
    const [inputQuery, setInputQuery] = useState("");
    const [activeTitle, setActiveTitle] = useState(""); // Prevents header flicker
    const [result, setResult] = useState<SearchResult | null>(null);
    const [isLoading, setIsLoading] = useState(false);

    const handleSearch = async (e?: React.FormEvent) => {
        if (e) e.preventDefault();
        if (!inputQuery.trim()) return;

        setIsLoading(true);
        setResult(null);
        setActiveTitle(inputQuery); // Lock in the title

        try {
            const response = await fetch("http://localhost:4000/search", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ query: inputQuery }),
            });

            const data = await response.json();

            // Parse the JSON string from Rust
            const parsed: SearchResult = JSON.parse(data.content);
            setResult(parsed);
        } catch (error) {
            console.error("Search failed:", error);
        } finally {
            setIsLoading(false);
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
                        {/* Header with Dynamic Category Tag */}
                        {/*<div className="flex items-baseline justify-between border-b border-neutral-700 pb-4">
                            <h1 className="segoe text-lg font-medium tracking-tight capitalize">
                                {activeTitle}
                            </h1>
                            <span className="rounded bg-orange-500/10 px-2 py-1 font-mono text-xs tracking-widest text-orange-500 uppercase">
                                {result.type}
                            </span>
                        </div>*/}

                        {/* --- BLOCK: WHO (Person) --- */}
                        {result.type === "who" && (
                            <div className="rounded-xl border-l-4 border-blue-500 bg-neutral-100 p-6 dark:bg-neutral-800">
                                <h2 className="text-xl font-bold">
                                    {result.name}
                                </h2>
                                <article className="mb-4 text-sm text-neutral-500">
                                    {result.lifespan}
                                </article>
                                <p className="mb-4 text-lg leading-relaxed">
                                    {result.known_for}
                                </p>
                                <div>
                                    <h3 className="mb-2 text-xs font-bold text-neutral-400 uppercase">
                                        Key Achievements
                                    </h3>
                                    <ul className="list-disc space-y-1 pl-5">
                                        {result.achievements.map((a, i) => (
                                            <li key={i}>{a}</li>
                                        ))}
                                    </ul>
                                </div>
                            </div>
                        )}

                        {/* --- BLOCK: HOW (Process) --- */}
                        {result.type === "how" && (
                            <div className="space-y-4">
                                <div className="flex justify-between text-sm text-neutral-500">
                                    <span>Process: {result.title}</span>
                                    <span>Difficulty: {result.difficulty}</span>
                                </div>
                                {result.steps.map((step) => (
                                    <div
                                        key={step.step}
                                        className="flex gap-4 rounded-lg bg-neutral-100 p-4 dark:bg-neutral-800"
                                    >
                                        <span className="text-2xl font-black text-neutral-300">
                                            #{step.step}
                                        </span>
                                        <p className="pt-1">
                                            {step.instruction}
                                        </p>
                                    </div>
                                ))}
                            </div>
                        )}

                        {/* --- BLOCK: WHAT (Concept) --- */}
                        {result.type === "what" && (
                            <div>
                                <h2 className="mb-4 w-full border-b border-neutral-300 pb-2 text-xl font-medium tracking-tighter dark:border-neutral-700">
                                    {result.concept}
                                </h2>
                                <article className="mb-6 text-lg">
                                    {result.definition}
                                </article>
                                <div className="grid grid-cols-2 gap-4">
                                    <div>
                                        <span className="mb-4 block w-full border-b border-neutral-300 pb-1 font-medium tracking-tight text-neutral-400 dark:border-neutral-700">
                                            Application
                                        </span>
                                        <article className="text-lg">
                                            {result.application}
                                        </article>
                                    </div>
                                    <div>
                                        <span className="mb-4 block w-full border-b border-neutral-300 pb-1 font-medium tracking-tight text-neutral-400 dark:border-neutral-700">
                                            Origin
                                        </span>
                                        <article className="text-lg">
                                            {result.origin}
                                        </article>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* --- BLOCK: WHEN (Event) --- */}
                        {result.type === "when" && (
                            <div className="rounded-xl border-t-4 border-purple-500 bg-neutral-100 p-6 dark:bg-neutral-800">
                                <div className="mb-4 flex items-center justify-between">
                                    <h2 className="text-xl font-bold">
                                        {result.event}
                                    </h2>
                                    <span className="rounded bg-purple-500/10 px-2 py-1 font-mono text-purple-500">
                                        {result.date}
                                    </span>
                                </div>
                                <p className="mb-6">{result.significance}</p>
                                <div className="relative ml-2 space-y-2 border-l border-neutral-600 pl-4">
                                    {result.timeline.map((t, i) => (
                                        <p
                                            key={i}
                                            className="text-sm text-neutral-400"
                                        >
                                            Before/After: {t}
                                        </p>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* --- BLOCK: WHERE (Location) --- */}
                        {result.type === "where" && (
                            <div className="relative overflow-hidden rounded-xl bg-neutral-100 p-6 dark:bg-neutral-800">
                                <h2 className="text-2xl font-bold">
                                    {result.location}
                                </h2>
                                <p className="mb-4 text-neutral-500">
                                    {result.region}
                                </p>

                                <div className="mb-4 flex flex-wrap gap-2">
                                    {result.facts.map((fact, i) => (
                                        <span
                                            key={i}
                                            className="rounded-full bg-white px-3 py-1 text-sm dark:bg-neutral-700"
                                        >
                                            {fact}
                                        </span>
                                    ))}
                                </div>
                                <div className="mt-4 rounded-lg bg-blue-500/10 p-4 text-sm text-blue-400">
                                    Climate/Vibe: {result.climate}
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
