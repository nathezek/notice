"use client";

import { motion, AnimatePresence } from "motion/react";
import { WebsiteData } from "@/mock_data/website_mock_data";
import { IconCopy, IconLock, IconX } from "@tabler/icons-react";

interface WebsiteModalProps {
    isOpen: boolean;
    onClose: () => void;
    website: WebsiteData | null;
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
                <div className="fixed inset-0 z-50 flex items-center justify-center">
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={onClose}
                        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
                    />

                    {/* Modal Content */}
                    <motion.div
                        initial={{ scale: 0.9, opacity: 0, y: 20 }}
                        animate={{ scale: 1, opacity: 1, y: 0 }}
                        exit={{ scale: 0.9, opacity: 0, y: 20 }}
                        transition={{
                            type: "spring",
                            damping: 25,
                            stiffness: 300,
                        }}
                        className="relative z-10 flex h-[92vh] w-[90vw] flex-col overflow-hidden rounded-xs bg-white shadow-2xl border dark:border-neutral-700 dark:bg-neutral-900"
                    >
                        {/* Header */}
                        <div className="flex items-center justify-between border-b border-neutral-200 bg-neutral-50 px-4 py-1 dark:bg-neutral-800 dark:border-neutral-700">
                            <div />
                            <div className="text-xs h-7 dark:bg-neutral-700/60 bg-blue-100 text-blue-500 dark:text-blue-300 rounded-md flex items-center justify-center max-w-2xl px-2 gap-x-4">
                                <IconLock size={14} />
                                {website.title}
                                <IconCopy size={14} />
                            </div>
                            <button
                                onClick={onClose}
                                className="rounded-sm p-1 transition-colors hover:bg-neutral-100 dark:hover:bg-neutral-700/50 cursor-pointer"
                            >
                                <IconX size={18} />
                            </button>
                        </div>

                        {/* Body - using iframe for now as requested "popup inside the site" */}
                        <div className="flex-1 bg-white">
                            <iframe
                                src={website.url}
                                title={website.title}
                                className="h-full w-full border-0"
                                sandbox="allow-same-origin allow-scripts"
                            />
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};
