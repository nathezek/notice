"use client";

import { useRouter, usePathname, useSearchParams } from "next/navigation";
import { useEffect, useRef } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useSearchStore } from "@/stores/search_store";
import { QuerySearchForm } from "../forms/query_search_form";
import { ThemeSwitcher } from "@/theme/theme_switcher";
import { useAuth } from "@/lib/auth";
import Link from "next/link";

export const Navbar = () => {
    const {
        inputQuery,
        setInputQuery,
        setResult,
        setLoading,
        result,
        isLoading,
    } = useSearchStore();

    const { user, logout } = useAuth();
    const router = useRouter();
    const pathname = usePathname();

    const isSearchPage = pathname === "/search";
    const hasSearched = isSearchPage || isLoading || result !== null;

    const lastPushedQuery = useRef<string | null>(null);

    const handleSearch = async (e?: React.FormEvent) => {
        if (e) e.preventDefault();

        const trimmed = inputQuery.trim();
        if (!trimmed) return;
        if (trimmed === lastPushedQuery.current) return;

        lastPushedQuery.current = trimmed;
        setTimeout(() => {
            lastPushedQuery.current = null;
        }, 1000);

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
            className="fixed top-0 z-50 flex w-full items-center justify-center overflow-hidden bg-white/80 backdrop-blur-md dark:bg-neutral-900/80"
        >
            <AnimatePresence mode="wait">
                {!hasSearched && (
                    <motion.div
                        key="home-layout"
                        className="mx-auto flex w-full max-w-2xl flex-col items-center justify-center gap-8"
                        initial={false}
                        exit={{ opacity: 0, transition: { duration: 0.2 } }}
                    >
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
                            className="ed-italic cursor-pointer text-6xl font-bold tracking-tighter text-neutral-900 dark:text-neutral-100"
                            onClick={handleLogoClick}
                        >
                            notice
                        </motion.h1>

                        <div className="w-full px-4">
                            <QuerySearchForm
                                handleSearch={handleSearch}
                                inputQuery={inputQuery}
                                setInputQuery={setInputQuery}
                            />
                        </div>

                        <div className="flex items-center gap-6 text-sm font-medium">
                            {user ? (
                                <div className="flex items-center gap-4">
                                    <span className="text-neutral-500">
                                        Hi, {user.username}
                                    </span>
                                    <button
                                        onClick={logout}
                                        className="transition-colors hover:text-indigo-500"
                                    >
                                        Logout
                                    </button>
                                </div>
                            ) : (
                                <div className="flex items-center gap-4">
                                    <Link
                                        href="/login"
                                        className="transition-colors hover:text-indigo-500"
                                    >
                                        Login
                                    </Link>
                                    <Link
                                        href="/register"
                                        className="rounded-full bg-neutral-900 px-4 py-2 text-white transition-opacity hover:opacity-80 dark:bg-white dark:text-neutral-900"
                                    >
                                        Sign Up
                                    </Link>
                                </div>
                            )}
                            <ThemeSwitcher />
                        </div>
                    </motion.div>
                )}

                {hasSearched && (
                    <motion.div
                        key="search-layout"
                        className="flex w-full items-center justify-between px-8"
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1, transition: { duration: 0.3 } }}
                        exit={{ opacity: 0, transition: { duration: 0.15 } }}
                    >
                        <motion.h1
                            className="w-24 shrink-0 cursor-pointer text-3xl font-bold tracking-tighter text-neutral-900 italic dark:text-neutral-100"
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

                        <div className="max-w-2xl flex-1 px-4">
                            <QuerySearchForm
                                handleSearch={handleSearch}
                                inputQuery={inputQuery}
                                setInputQuery={setInputQuery}
                            />
                        </div>

                        <div className="flex w-48 shrink-0 items-center justify-end gap-4">
                            {user && (
                                <span className="hidden truncate text-xs text-neutral-500 md:block">
                                    {user.username}
                                </span>
                            )}
                            <ThemeSwitcher />
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </motion.nav>
    );
};

export default Navbar;
