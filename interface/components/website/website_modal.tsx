"use client";

import { motion, AnimatePresence } from "motion/react";
import { IconCopy, IconLock, IconX, IconExternalLink } from "@tabler/icons-react";

export interface WebsiteMetadata {
    url: string;
    title: string;
    description?: string;
    favicon?: string;
}

interface WebsiteModalProps {
    isOpen: boolean;
    onClose: () => void;
    website: WebsiteMetadata | null;
}

export const WebsiteModal = ({
    isOpen,
    onClose,
    website,
}: WebsiteModalProps) => {
    if (!website) return null;

    return (
        <AnimatePresence>
            {isOpen && (
                <div className="fixed inset-0 z-100 flex items-center justify-center p-4 md:p-10">
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={onClose}
                        className="absolute inset-0 bg-black/40 backdrop-blur-sm"
                    />

                    {/* Modal Content */}
                    <motion.div
                        initial={{ scale: 0.95, opacity: 0, y: 20 }}
                        animate={{ scale: 1, opacity: 1, y: 0 }}
                        exit={{ scale: 0.95, opacity: 0, y: 20 }}
                        transition={{
                            type: "spring",
                            damping: 30,
                            stiffness: 300,
                        }}
                        className="relative z-10 flex h-full w-full flex-col overflow-hidden rounded-xl border border-neutral-200 bg-white shadow-2xl dark:border-neutral-800 dark:bg-neutral-900"
                    >
                        {/* Header / Browser-like address bar */}
                        <div className="flex items-center justify-between border-b border-neutral-100 bg-neutral-50/50 p-3 dark:border-neutral-800 dark:bg-neutral-900/50">
                            <div className="flex items-center gap-2">
                                <div className="flex gap-1.5 px-2">
                                    <div className="h-3 w-3 rounded-full bg-red-400/20 dark:bg-red-400/40" />
                                    <div className="h-3 w-3 rounded-full bg-yellow-400/20 dark:bg-yellow-400/40" />
                                    <div className="h-3 w-3 rounded-full bg-green-400/20 dark:bg-green-400/40" />
                                </div>
                            </div>

                            <div className="flex h-9 min-w-[300px] max-w-xl flex-1 items-center gap-x-3 rounded-lg border border-neutral-200 bg-white px-4 text-xs text-neutral-500 shadow-sm dark:border-neutral-800 dark:bg-neutral-950 dark:text-neutral-400">
                                <IconLock size={14} className="shrink-0 text-green-500/70" />
                                <span className="flex-1 truncate font-medium text-neutral-600 dark:text-neutral-300">
                                    {website.url}
                                </span>
                                <div className="flex gap-2 shrink-0">
                                    <button onClick={() => window.open(website.url, '_blank')} className="hover:text-blue-500 transition-colors">
                                        <IconExternalLink size={14} />
                                    </button>
                                    <button onClick={() => navigator.clipboard.writeText(website.url)} className="hover:text-blue-500 transition-colors">
                                        <IconCopy size={14} />
                                    </button>
                                </div>
                            </div>

                            <button
                                onClick={onClose}
                                className="ml-4 cursor-pointer rounded-full p-2 text-neutral-400 transition-all hover:bg-neutral-100 hover:text-neutral-600 dark:hover:bg-neutral-800 dark:hover:text-neutral-200"
                            >
                                <IconX size={20} />
                            </button>
                        </div>

                        {/* Body - using iframe */}
                        <div className="flex-1 bg-white dark:bg-white overflow-hidden">
                            <iframe
                                src={website.url}
                                title={website.title}
                                className="h-full w-full border-0"
                                sandbox="allow-same-origin allow-scripts allow-popups allow-forms"
                            />
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};
