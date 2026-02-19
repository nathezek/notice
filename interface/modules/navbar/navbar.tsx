"use client";

import { motion } from "motion/react";
import { useSearchStore } from "@/stores/search_store";
import { QuerySearchForm } from "../forms/query_search_form";
import React from "react";

export const Navbar = () => {
    const {
        inputQuery,
        setInputQuery,
        setResult,
        setLoading,
        result,
        isLoading,
    } = useSearchStore();

    const hasSearched = isLoading || result !== null;

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
            setResult(JSON.parse(data.content));
        } catch (error) {
            console.error("Search failed:", error);
        } finally {
            setLoading(false);
        }
    };

    return (
        <motion.nav
            // 1. Slow entrance duration on first load
            initial={{ height: 0, opacity: 0 }}
            animate={{
                height: hasSearched ? 80 : 450,
                opacity: 1,
            }}
            transition={{
                height: {
                    type: "spring",
                    stiffness: 100,
                    damping: 20,
                    duration: 1.5,
                },
                opacity: { duration: 1 },
            }}
            className="fixed top-0 z-50 flex w-full items-center justify-center border-b border-neutral-200/0 bg-white/0 backdrop-blur-none dark:border-neutral-800/0 dark:bg-black/0"
        >
            <motion.div
                layout
                className={`flex w-full items-center gap-8 transition-all duration-700 ${
                    hasSearched
                        ? "max-w-5xl flex-row justify-between px-8"
                        : "flex-col justify-center lg:w-2xl"
                }`}
            >
                {/* 2 & 3. sequenced h1 opacity */}
                <motion.h1
                    layout
                    initial={{ opacity: 0 }}
                    animate={{
                        opacity: 1,
                        fontSize: hasSearched ? "2rem" : "4rem",
                        marginBottom: hasSearched ? 0 : 32,
                        // Dip the opacity during the morph
                        filter: hasSearched ? "blur(0px)" : "blur(0px)",
                    }}
                    transition={{
                        opacity: {
                            delay: hasSearched ? 0 : 0.8,
                            duration: 0.8,
                        },
                        layout: { duration: 0.8, type: "spring", bounce: 0.2 },
                        fontSize: { duration: 0.6 },
                    }}
                    className="ed-italic font-bold tracking-tighter"
                >
                    notice
                </motion.h1>

                <motion.div
                    layout
                    className={hasSearched ? "flex-1" : "w-full"}
                >
                    <QuerySearchForm
                        handleSearch={handleSearch}
                        inputQuery={inputQuery}
                        setInputQuery={setInputQuery}
                    />
                </motion.div>
            </motion.div>
        </motion.nav>
    );
};
