"use client";

import { useState } from "react";
import { WebsiteData } from "@/mock_data/website_mock_data";
import { motion, AnimatePresence } from "motion/react";

interface WebsiteListProps {
    websites: WebsiteData[];
    onWebsiteClick: (website: WebsiteData) => void;
}

export const WebsiteList = ({ websites, onWebsiteClick }: WebsiteListProps) => {
    const [hoveredId, setHoveredId] = useState<string | null>(null);

    return (
        <div className="flex h-full w-full flex-col gap-y-4">
            <h3 className="text-sm font-medium opacity-50">Sources</h3>
            <ul className="space-y-3">
                {websites.map((site) => (
                    <li
                        key={site.id}
                        className="group relative"
                        onMouseEnter={() => setHoveredId(site.id)}
                        onMouseLeave={() => setHoveredId(null)}
                    >
                        {/* List Item Content */}
                        <button
                            onClick={() => onWebsiteClick(site)}
                            className="flex w-full flex-col items-start gap-1 rounded-lg border border-neutral-200 bg-white p-3 text-left transition-colors hover:bg-neutral-50 dark:border-neutral-800 dark:bg-neutral-900 dark:hover:bg-neutral-800"
                        >
                            <div className="flex items-center gap-2">
                                <img
                                    src={`https://www.google.com/s2/favicons?domain=${site.url}&sz=32`}
                                    alt=""
                                    className="h-4 w-4 rounded-sm"
                                />
                                <span className="line-clamp-1 text-sm font-medium text-blue-600 hover:underline dark:text-blue-400">
                                    {site.title}
                                </span>
                            </div>
                            <p className="line-clamp-2 text-xs text-neutral-600 dark:text-neutral-400">
                                {site.snippet}
                            </p>
                        </button>

                        {/* Hover Preview Tooltip */}
                        <AnimatePresence>
                            {hoveredId === site.id && (
                                <motion.div
                                    initial={{
                                        opacity: 0,
                                        x: -10,
                                        scale: 0.95,
                                    }}
                                    animate={{ opacity: 1, x: 0, scale: 1 }}
                                    exit={{ opacity: 0, x: -10, scale: 0.95 }}
                                    transition={{ duration: 0.2 }}
                                    className="absolute top-0 right-full z-50 mr-4 w-64 rounded-xl border border-neutral-200 bg-white p-2 shadow-xl dark:border-neutral-800 dark:bg-neutral-950"
                                >
                                    {/* Arrow */}
                                    <div className="absolute top-4 right-1.5 h-3 w-3 rotate-45 border-t border-r border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-950" />

                                    <div className="overflow-hidden rounded-lg">
                                        <div className="h-32 w-full bg-neutral-100 dark:bg-neutral-800">
                                            {/* Ideally real screenshot, using placeholder or mock image */}
                                            <img
                                                src={
                                                    site.imageUrl ||
                                                    "https://placehold.co/400x300"
                                                }
                                                alt={site.title}
                                                className="h-full w-full object-cover"
                                            />
                                        </div>
                                        <div className="p-3">
                                            <h4 className="mb-1 text-sm font-semibold">
                                                {site.title}
                                            </h4>
                                            <p className="text-xs text-neutral-500">
                                                {site.url}
                                            </p>
                                        </div>
                                    </div>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </li>
                ))}
            </ul>
        </div>
    );
};
