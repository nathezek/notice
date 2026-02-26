import type { SearchResult } from "@/lib/api";

interface Props {
    results: SearchResult[];
    total: number;
    query: string;
}

export default function SearchResults({ results, total, query }: Props) {
    if (results.length === 0) {
        return (
            <div className="py-12 text-center">
                <p className="text-lg">
                    No results found for &ldquo;{query}&rdquo;
                </p>
                <p className="mt-2 text-sm">
                    Try different keywords or submit a URL to index
                </p>
            </div>
        );
    }

    return (
        <div>
            <p className="mb-4 text-sm">
                {total} result{total !== 1 ? "s" : ""}
            </p>

            <div className="space-y-5">
                {results.map((result) => (
                    <ResultCard key={result.id} result={result} />
                ))}
            </div>
        </div>
    );
}

function ResultCard({ result }: { result: SearchResult }) {
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
            {/* URL */}
            <div className="mb-1 flex items-center gap-2">
                <span className="result-url max-w-md truncate">
                    {displayUrl}
                </span>
                {result.score !== null && (
                    <span className="score-badge">
                        {(result.score * 100).toFixed(0)}%
                    </span>
                )}
            </div>

            {/* Title */}
            <h3 className="mb-1">
                <a
                    href={result.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="result-title text-lg font-medium"
                >
                    {result.title || result.url}
                </a>
            </h3>

            {/* Snippet */}
            <p className="line-clamp-2 text-sm leading-relaxed">
                {result.snippet}
            </p>
        </article>
    );
}
