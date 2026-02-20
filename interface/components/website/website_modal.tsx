"use client";

import { motion, AnimatePresence } from "motion/react";
import { WebsiteData } from "@/mock_data/website_mock_data";
import { IconX } from "@tabler/icons-react";

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
                        className="relative z-10 flex h-[95vh] w-[95vw] flex-col overflow-hidden rounded-xs bg-white shadow-2xl dark:bg-neutral-900"
                    >
                        {/* Header */}
                        <div className="flex items-center justify-between border-b border-neutral-200 bg-neutral-50 px-4 py-1 dark:border-neutral-800 dark:bg-neutral-950">
                            <h2 className="text-base font-medium text-neutral-900 dark:text-neutral-100">
                                {website.title}
                            </h2>
                            <button
                                onClick={onClose}
                                className="rounded-full p-2 transition-colors hover:bg-neutral-200 dark:hover:bg-neutral-800"
                            >
                                <IconX size={20} />
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
