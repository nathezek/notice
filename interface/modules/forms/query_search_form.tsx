import { IconSearch, IconX } from "@tabler/icons-react";
import React from "react";

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
    return (
        <form
            className="relative flex h-14 w-full max-w-2xl items-center justify-between gap-x-2 rounded-xl bg-neutral-200 px-4 dark:bg-neutral-800"
            onSubmit={handleSearch}
        >
            <IconSearch className="text-neutral-500" size={20} />
            <input
                type="text"
                value={inputQuery}
                className="flex-1 bg-transparent p-2 font-sans outline-none" // Using font-sans for UI as discussed
                placeholder="Search"
                onChange={(e) => setInputQuery(e.target.value)}
            />
            {inputQuery && (
                <button
                    type="button"
                    onClick={() => setInputQuery("")}
                    className="transition-opacity hover:opacity-70"
                >
                    <IconX size={18} className="text-neutral-500" />
                </button>
            )}
        </form>
    );
};
