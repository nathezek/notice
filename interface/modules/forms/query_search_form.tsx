import { IconSearch, IconX } from "@tabler/icons-react";
import React, { useEffect, useRef } from "react";
import { useSearchStore } from "@/stores/search_store";

type T_QUERY_SEARCH_FORM = {
    inputQuery: string;
    setInputQuery: (value: string) => void;
    handleSearch: (e: React.SubmitEvent) => void;
};

export const QuerySearchForm = ({
    handleSearch,
    inputQuery,
    setInputQuery,
}: T_QUERY_SEARCH_FORM) => {
    const inputRef = useRef<HTMLInputElement>(null);
    const { isModalOpen, setModalOpen } = useSearchStore();

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "/" && document.activeElement?.tagName !== "INPUT" && document.activeElement?.tagName !== "TEXTAREA") {
                e.preventDefault();
                if (isModalOpen) {
                    setModalOpen(false);
                }
                inputRef.current?.focus();
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [isModalOpen, setModalOpen]);

    const handleClear = () => {
        setInputQuery("");
        inputRef.current?.focus();
    };

    return (
        <form
            className="relative flex h-11 w-full items-center justify-between gap-x-2 rounded-2xl bg-neutral-200/50 px-4 backdrop-blur-xl dark:bg-neutral-800/80"
            onSubmit={handleSearch}
        >
            <IconSearch className="text-neutral-500" size={20} />
            <input
                ref={inputRef}
                type="text"
                value={inputQuery}
                className="h-full flex-1 truncate bg-transparent px-2 font-medium tracking-tight outline-none"
                placeholder="Search"
                onChange={(e) => setInputQuery(e.target.value)}
            />
            {inputQuery && (
                <button
                    type="button"
                    onClick={handleClear}
                    className="h-full cursor-pointer px-2 transition-opacity hover:opacity-70"
                >
                    <IconX size={18} className="text-neutral-500" />
                </button>
            )}
        </form>
    );
};
