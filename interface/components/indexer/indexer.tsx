"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Globe, Plus, Check, Loader2, AlertCircle } from "lucide-react";

export const IndexerComponent = () => {
    const [url, setUrl] = useState("");
    const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
    const [message, setMessage] = useState("");

    const handleIndex = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!url || !url.startsWith("http")) {
            setStatus("error");
            setMessage("Please enter a valid URL starting with http");
            return;
        }

        setStatus("loading");
        try {
            const response = await fetch("http://localhost:4000/index-url", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ url }),
            });
            const data = await response.json();

            if (data.success) {
                setStatus("success");
                setMessage("Successfully added to your Vault.");
                setUrl("");
                setTimeout(() => setStatus("idle"), 3000);
            } else {
                setStatus("error");
                setMessage(data.message || "Failed to index URL.");
            }
        } catch (error) {
            setStatus("error");
            setMessage("Connection to indexer failed.");
        }
    };

    return (
        <div className="w-full max-w-md rounded-3xl border border-neutral-200 bg-white/80 p-6 shadow-xl backdrop-blur-xl dark:border-neutral-800 dark:bg-neutral-900/80">
            <div className="mb-4 flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-blue-500/10 text-blue-500">
                    <Globe size={20} />
                </div>
                <div>
                    <h3 className="text-sm font-bold text-neutral-900 dark:text-white">Knowledge Vault</h3>
                    <p className="text-xs text-neutral-500">Crawl new pages into your index</p>
                </div>
            </div>

            <form onSubmit={handleIndex} className="relative flex flex-col gap-3">
                <div className="group relative flex items-center">
                    <input
                        type="text"
                        placeholder="https://example.com"
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                        disabled={status === "loading"}
                        className="w-full rounded-2xl border border-neutral-200 bg-neutral-50 py-3 pl-4 pr-12 text-sm outline-none transition-all focus:border-blue-500 focus:bg-white focus:ring-4 focus:ring-blue-500/10 dark:border-neutral-800 dark:bg-neutral-950 dark:focus:border-blue-500 dark:focus:bg-neutral-900"
                    />
                    <button
                        type="submit"
                        disabled={status === "loading" || !url}
                        className="absolute right-2 flex h-9 w-9 items-center justify-center rounded-xl bg-neutral-900 text-white transition-all hover:bg-neutral-800 disabled:opacity-50 dark:bg-neutral-100 dark:text-black dark:hover:bg-neutral-200"
                    >
                        {status === "loading" ? (
                            <Loader2 size={18} className="animate-spin" />
                        ) : (
                            <Plus size={18} />
                        )}
                    </button>
                </div>

                <AnimatePresence mode="wait">
                    {status !== "idle" && (
                        <motion.div
                            initial={{ opacity: 0, height: 0 }}
                            animate={{ opacity: 1, height: "auto" }}
                            exit={{ opacity: 0, height: 0 }}
                            className="overflow-hidden"
                        >
                            <div className={`mt-2 flex items-center gap-2 rounded-xl p-3 text-xs font-medium ${status === "success" ? "bg-green-500/10 text-green-600 dark:bg-green-500/20 dark:text-green-400" :
                                    status === "error" ? "bg-red-500/10 text-red-600 dark:bg-red-500/20 dark:text-red-400" :
                                        "bg-blue-500/10 text-blue-600 dark:bg-blue-500/20 dark:text-blue-400"
                                }`}>
                                {status === "success" && <Check size={14} />}
                                {status === "error" && <AlertCircle size={14} />}
                                {status === "loading" && <Loader2 size={14} className="animate-spin" />}
                                <span>{status === "loading" ? "Crawl in progress... summarizing with Gemini" : message}</span>
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </form>

            <div className="mt-6 flex flex-wrap gap-2 opacity-50 transition-opacity hover:opacity-100">
                <span className="text-[10px] uppercase tracking-wider text-neutral-400">Trusted Sources</span>
                <div className="flex gap-3 grayscale transition-all hover:grayscale-0">
                    <div className="text-[10px] font-bold">WIKIPEDIA</div>
                    <div className="text-[10px] font-bold">MDN</div>
                    <div className="text-[10px] font-bold">GITHUB</div>
                </div>
            </div>
        </div>
    );
};
