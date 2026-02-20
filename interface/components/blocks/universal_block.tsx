"use client";

import { Article } from "@/components/ui/article";
import { IconMapPin, IconInfoCircle } from "@tabler/icons-react";
import ReactMarkdown from "react-markdown";

export interface UniversalResultData {
    summary: string;
    facts?: { label: string; value: string }[];
    related_topics?: string[];
    widgets?: { type: string; query: string }[];
}

export const UniversalBlock = ({ data }: { data: UniversalResultData }) => {
    return (
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-700">
            {/* 1. Main Summary (Article) */}
            <Article>
                <div className="prose prose-neutral dark:prose-invert max-w-none leading-relaxed text-neutral-800 dark:text-neutral-200">
                    <ReactMarkdown>{data.summary}</ReactMarkdown>
                </div>
            </Article>

            {/* 2. Facts Grid */}
            {data.facts && data.facts.length > 0 && (
                <div className="mt-8 grid grid-cols-1 gap-4 rounded-xl bg-neutral-50 p-6 dark:bg-neutral-800/50 md:grid-cols-2">
                    {data.facts.map((fact, i) => (
                        <div key={i} className="flex flex-col">
                            <span className="text-xs font-semibold uppercase tracking-wider text-neutral-500">
                                {fact.label}
                            </span>
                            <span className="text-lg font-medium text-neutral-800 dark:text-neutral-100">
                                {fact.value}
                            </span>
                        </div>
                    ))}
                </div>
            )}

            {/* 3. Related Topics */}
            {data.related_topics && data.related_topics.length > 0 && (
                <div className="mt-8">
                    <h3 className="mb-3 text-sm font-semibold text-neutral-900 dark:text-neutral-100">
                        Explore More
                    </h3>
                    <div className="flex flex-wrap gap-2">
                        {data.related_topics.map((topic, i) => (
                            <a
                                key={i}
                                href={`/search?query=${encodeURIComponent(topic)}`}
                                className="rounded-full border border-neutral-200 bg-white px-4 py-1.5 text-sm text-neutral-600 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                            >
                                {topic}
                            </a>
                        ))}
                    </div>
                </div>
            )}

            {/* 4. Widgets (Simple placeholder for now) */}
            {data.widgets?.map((widget, i) => (
                <div key={i} className="mt-8 rounded-xl border border-blue-100 bg-blue-50 p-4 dark:border-blue-900/30 dark:bg-blue-900/10">
                    <div className="flex items-center gap-2 text-blue-700 dark:text-blue-300">
                        <IconMapPin size={20} />
                        <span className="font-medium">Map: {widget.query}</span>
                    </div>
                    {/* Placeholder for map */}
                    <div className="mt-3 h-48 w-full rounded-lg bg-blue-200/50 dark:bg-blue-800/30" />
                </div>
            ))}
        </div>
    );
};
