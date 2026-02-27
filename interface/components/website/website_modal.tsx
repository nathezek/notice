"use client";

import { motion, AnimatePresence } from "motion/react";
import {
    IconCopy,
    IconLock,
    IconX,
    IconExternalLink,
} from "@tabler/icons-react";

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
                <div className="fixed inset-0 z-100 flex items-center justify-center p-4 pt-9 md:px-7">
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={onClose}
                        className="absolute inset-0 bg-black/40 backdrop-blur-[2px]"
                    />

                    {/* Modal Content */}
                    <motion.div
                        initial={{ scale: 0.95, opacity: 0, y: 20 }}
                        animate={{ scale: 1, opacity: 1, y: 0 }}
                        exit={{ scale: 0.9, opacity: 0, y: 20 }}
                        transition={{
                            type: "spring",
                            duration: 0.55,
                            bounce: 0.4,
                        }}
                        className="relative z-10 flex h-full w-full flex-col overflow-hidden rounded-md border border-neutral-200 bg-white shadow-2xl dark:border-neutral-700 dark:bg-neutral-900"
                    >
                        {/* Header / Browser-like address bar */}
                        <div className="flex h-10 items-center justify-between border-b border-neutral-100 bg-neutral-50/50 p-3 dark:border-neutral-700 dark:bg-neutral-900/50">
                            <div />

                            <div className="flex h-7 max-w-xl min-w-75 flex-1 items-center gap-x-3 rounded-lg bg-neutral-100 px-4 text-xs dark:bg-neutral-700/30">
                                <IconLock
                                    size={14}
                                    className="shrink-0 text-blue-500/90"
                                />
                                <span className="flex-1 truncate font-medium text-blue-500 dark:text-blue-200">
                                    {website.url}
                                </span>
                                <div className="flex shrink-0 gap-2">
                                    <button
                                        onClick={() =>
                                            window.open(website.url, "_blank")
                                        }
                                        className="cursor-pointer transition-colors hover:text-blue-500"
                                    >
                                        <IconExternalLink size={14} />
                                    </button>
                                    <button
                                        onClick={() =>
                                            navigator.clipboard.writeText(
                                                website.url,
                                            )
                                        }
                                        className="cursor-pointer transition-colors hover:text-blue-500"
                                    >
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
                        <div className="flex-1 overflow-hidden bg-white dark:bg-white">
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
