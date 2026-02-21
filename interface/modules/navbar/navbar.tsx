"use client";

import { useRouter, usePathname, useSearchParams } from "next/navigation";
import { useEffect } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useSearchStore } from "@/stores/search_store";
import { QuerySearchForm } from "../forms/query_search_form";
import { ThemeSwitcher } from "@/theme/theme_switcher";
import React, { useRef } from "react";

export const Navbar = () => {
    const {
        inputQuery,
        setInputQuery,
        setResult,
        setLoading,
        result,
        isLoading,
    } = useSearchStore();

    const router = useRouter();
    const pathname = usePathname();
    const searchParams = useSearchParams();

    const isSearchPage = pathname === "/search";
    const hasSearched = isSearchPage || isLoading || result !== null;

    useEffect(() => {
        if (pathname === "/") {
            // resetSearch(); // optional
        }
    }, [pathname]);

    const lastPushedQuery = useRef<string | null>(null);

    const handleSearch = async (e?: React.SubmitEvent) => {
        if (e) e.preventDefault();

        const trimmed = inputQuery.trim();
        if (!trimmed) return;
        if (trimmed === lastPushedQuery.current) return;

        lastPushedQuery.current = trimmed;
        // Reset the ref after a short delay to allow re-searching the same thing later intentionally
        setTimeout(() => { lastPushedQuery.current = null; }, 1000);

        router.push(`/search?query=${encodeURIComponent(trimmed)}`);
    };

    const handleLogoClick = () => {
        router.push("/");
        setInputQuery("");
        setResult(null);
        setLoading(false);
    };

    return (
        <motion.nav
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: hasSearched ? 80 : 450, opacity: 1 }}
            transition={{
                type: "spring",
                duration: 1,
                bounce: 0.3,
            }}
            className="fixed top-0 z-50 flex w-full items-center justify-center overflow-hidden"
        >
            {/* ---- HOME STATE: Large centered title + form ---- */}
            <AnimatePresence mode="wait">
                {!hasSearched && (
                    <motion.div
                        key="home-layout"
                        className="mx-auto flex w-full max-w-2xl flex-col items-center justify-center gap-8"
                        initial={false}
                        exit={{ opacity: 0, transition: { duration: 0.2 } }}
                    >
                        {/* Only the h1 fades in on mount */}
                        <motion.h1
                            initial={{ opacity: 0, y: 10 }}
                            animate={{
                                opacity: 1,
                                y: 0,
                                transition: {
                                    delay: 0.5,
                                    type: "spring",
                                    bounce: 0.2,
                                    duration: 0.7,
                                },
                            }}
                            exit={{
                                opacity: 0,
                                y: -10,
                                transition: { duration: 0.2 },
                            }}
                            className="ed-italic cursor-pointer text-6xl font-bold tracking-tighter text-neutral-900 dark:text-neutral-100"
                            onClick={handleLogoClick}
                        >
                            notice
                        </motion.h1>

                        <div className="w-full">
                            <QuerySearchForm
                                handleSearch={handleSearch}
                                inputQuery={inputQuery}
                                setInputQuery={setInputQuery}
                            />
                        </div>
                    </motion.div>
                )}

                {/* ---- SEARCH STATE: Compact bar ---- */}
                {hasSearched && (
                    <motion.div
                        key="search-layout"
                        className="flex w-full items-center justify-between px-8"
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1, transition: { duration: 0.3 } }}
                        exit={{ opacity: 0, transition: { duration: 0.15 } }}
                    >
                        {/* Left: small logo — fixed width to balance the right side */}
                        <motion.h1
                            className="ed-italic w-24 shrink-0 cursor-pointer text-3xl font-bold tracking-tighter text-neutral-900 dark:text-neutral-100"
                            initial={{ opacity: 0, y: -10 }}
                            animate={{
                                opacity: 1,
                                y: 0,
                                transition: {
                                    type: "spring",
                                    stiffness: 120,
                                    damping: 18,
                                },
                            }}
                            onClick={handleLogoClick}
                        >
                            notice
                        </motion.h1>

                        {/* Center: search form */}
                        <div className="flex-1 px-4">
                            <QuerySearchForm
                                handleSearch={handleSearch}
                                inputQuery={inputQuery}
                                setInputQuery={setInputQuery}
                            />
                        </div>

                        {/* Right: theme switcher — same fixed width as the logo to keep form centered */}
                        <div className="flex w-24 shrink-0 justify-end">
                            <ThemeSwitcher />
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </motion.nav>
    );
};
