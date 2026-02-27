"use client";

import { useState } from "react";
import { api } from "@/lib/api";

export default function SubmitUrl() {
    const [url, setUrl] = useState("");
    const [status, setStatus] = useState<{
        type: "success" | "error" | "info";
        message: string;
    } | null>(null);
    const [loading, setLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const trimmed = url.trim();
        if (!trimmed) return;

        setLoading(true);
        setStatus(null);

        try {
            const res = await api.submitUrl(trimmed);

            const typeMap: Record<string, "success" | "info"> = {
                queued: "success",
                exists: "info",
                already_queued: "info",
            };

            setStatus({
                type: typeMap[res.status] || "success",
                message: res.message,
            });
            if (res.status === "queued") setUrl("");
        } catch (err) {
            setStatus({
                type: "error",
                message:
                    err instanceof Error ? err.message : "Submission failed",
            });
        } finally {
            setLoading(false);
        }
    };

    const statusColors = {
        success: "text-[var(--success)]",
        error: "text-[var(--error)]",
        info: "text-[var(--warning)]",
    };

    return (
        <div className="rounded-xl border p-4">
            <h3 className="mb-3 text-sm font-medium">Submit a URL to index</h3>

            <form onSubmit={handleSubmit} className="flex gap-2">
                <input
                    type="url"
                    value={url}
                    onChange={(e) => setUrl(e.target.value)}
                    placeholder="https://example.com/article"
                    className="flex-1 rounded-lg border px-3 py-2 text-sm transition-colors"
                />
                <button
                    type="submit"
                    disabled={loading || !url.trim()}
                    className="rounded-lg border px-4 py-2 text-sm transition-colors hover:text-white disabled:opacity-40"
                >
                    {loading ? "..." : "Submit"}
                </button>
            </form>

            {status && (
                <p className={`mt-2 text-xs ${statusColors[status.type]}`}>
                    {status.message}
                </p>
            )}
        </div>
    );
}
