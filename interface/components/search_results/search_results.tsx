import type { SearchResult } from "@/lib/api";
import Image from "next/image";

interface Props {
    results: SearchResult[];
    total: number;
    query: string;
    onResultClick?: (result: SearchResult) => void;
}

export default function SearchResults({ results, total, query, onResultClick }: Props) {
    if (results.length === 0) {
        return (
            <div className="py-12 text-center animate-in fade-in duration-500">
                <p className="text-lg text-neutral-500">
                    No results found for &ldquo;<span className="font-semibold text-neutral-900 dark:text-neutral-100">{query}</span>&rdquo;
                </p>
                <p className="mt-2 text-sm text-neutral-400">
                    Try different keywords or submit a URL to index
                </p>
            </div>
        );
    }

    return (
        <div className="animate-in fade-in duration-700">
            <p className="mb-6 text-xs font-medium text-neutral-400 uppercase tracking-widest">
                {total} result{total !== 1 ? "s" : ""} found
            </p>

            <div className="space-y-12">
                {results.map((result) => (
                    <ResultCard key={result.id} result={result} onResultClick={onResultClick} />
                ))}
            </div>
        </div>
    );
}

function ResultCard({ result, onResultClick }: { result: SearchResult; onResultClick?: (result: SearchResult) => void }) {
    const domain = (() => {
        try {
            return new URL(result.url).hostname;
        } catch {
            return "";
        }
    })();

    const displayUrl = (() => {
        try {
            const u = new URL(result.url);
            return u.hostname + (u.pathname !== "/" ? u.pathname : "");
        } catch {
            return result.url;
        }
    })();

    return (
        <article className="group">
            {/* Site Info */}
            <div className="mb-2 flex items-center gap-2">
                <div className="flex h-5 w-5 items-center justify-center rounded-sm bg-neutral-100 p-0.5 dark:bg-neutral-800">
                    <Image
                        src={`https://www.google.com/s2/favicons?domain=${domain}&sz=32`}
                        alt=""
                        width={16}
                        height={16}
                        className="shrink-0 rounded-xs"
                    />
                </div>
                <span className="max-w-md truncate text-xs text-neutral-500 font-medium">
                    {displayUrl}
                </span>
                {result.score !== null && (
                    <span className="text-[10px] font-bold text-indigo-400/80">
                        {(result.score * 100).toFixed(0)}%
                    </span>
                )}
            </div>

            {/* Title */}
            <h3 className="mb-2">
                <a
                    href={result.url}
                    onClick={(e) => {
                        if (onResultClick) {
                            e.preventDefault();
                            onResultClick(result);
                        }
                    }}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-xl font-semibold text-blue-600 hover:underline dark:text-blue-400 transition-colors decoration-blue-600/30 underline-offset-4"
                >
                    {result.title || result.url}
                </a>
            </h3>

            {/* Snippet */}
            <p className="line-clamp-3 text-sm leading-relaxed text-neutral-600 dark:text-neutral-400">
                {result.snippet}
            </p>
        </article>
    );
}
