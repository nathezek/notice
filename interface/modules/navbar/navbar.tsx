"use client";

import { motion } from "motion/react";
//state import
import { useSearchStore } from "@/stores/search_store";
import { usePathname } from "next/navigation";
// component imports
import { QuerySearchForm } from "../forms/query_search_form";

export const Navbar = () => {
    const pathname = usePathname();
    const { inputQuery, setInputQuery, setResult, setLoading } =
        useSearchStore();

    const handleSearch = async (e?: React.SubmitEvent) => {
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
        <motion.nav
            initial={{ height: 10, opacity: 0 }}
            animate={{ height: 400, opacity: 100 }}
            transition={{
                type: "spring",
                stiffness: 210,
                damping: 20,
                duration: 3,
            }}
            className="fixed top-0 flex w-full items-center justify-center opacity-0"
        >
            <div className="flex w-full flex-col items-center lg:w-2xl">
                {pathname == "/" && (
                    <h1 className="ed-italic mb-8 text-6xl font-bold">
                        notice
                    </h1>
                )}

                {/* --- Search Bar --- */}
                <QuerySearchForm
                    handleSearch={handleSearch}
                    inputQuery={inputQuery}
                    setInputQuery={setInputQuery}
                />
            </div>
        </motion.nav>
    );
};
