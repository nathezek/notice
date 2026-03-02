"use client";
import { Article } from "../ui/article";
import ReactMarkdown from "react-markdown";

interface AiAnswerData {
    title: string;
    summary: string;
}

interface Props {
    title?: string;
    answer: string;
    isLoading?: boolean;
}

export default function Summary({ title, answer, isLoading }: Props) {
    // If we have an explicit title and it's loading, show a skeleton
    if (isLoading && !answer) {
        return (
            <div className="animate-pulse w-full">
                <div className="mb-4 h-8 w-48 rounded bg-neutral-200 dark:bg-neutral-800"></div>
                <div className="space-y-3">
                    <div className="h-4 w-full rounded bg-neutral-100 dark:bg-neutral-800/50"></div>
                    <div className="h-4 w-5/6 rounded bg-neutral-100 dark:bg-neutral-800/50"></div>
                    <div className="h-4 w-4/6 rounded bg-neutral-100 dark:bg-neutral-800/50"></div>
                </div>
            </div>
        );
    }

    if (!answer) return null;

    // Attempt to parse as JSON for backward compatibility or if backend didn't normalize
    let data: AiAnswerData;
    try {
        const cleanJson = (str: string) => {
            const match =
                str.match(/```json\n([\s\S]*?)\n```/) ||
                str.match(/```([\s\S]*?)```/);
            return match ? match[1].trim() : str.trim();
        };

        const cleaned = cleanJson(answer);
        data = JSON.parse(cleaned);
    } catch (e) {
        data = {
            title: title || "Overview",
            summary: answer,
        };
    }

    // Convert standalone bold lines to h3 for proper section styling
    const processedSummary = data.summary.replace(
        /^[\s]*(?:\*\*|__)(.*?)(?:\*\*|__)(:?)[\s]*$/gm,
        "### $1$2",
    );

    return (
        <article className="animate-in fade-in slide-in-from-bottom-4 w-full duration-700">
            <div className="mb-3 flex items-center gap-2 border-b border-neutral-200 pb-1 dark:border-neutral-800">
                <span className="text-2xl font-medium tracking-tight text-neutral-800 dark:text-neutral-200">
                    {data.title || title || "Overview"}
                </span>
                {isLoading && (
                    <div className="h-3 w-3 animate-spin rounded-full border-2 border-neutral-300 border-t-neutral-800 dark:border-neutral-700 dark:border-t-neutral-200"></div>
                )}
            </div>

            <Article>
                <ReactMarkdown>{processedSummary}</ReactMarkdown>
            </Article>

            <div className="mt-4 font-sans flex items-center gap-2 border-t border-neutral-100 pt-3 text-[10px] text-neutral-400 dark:border-neutral-800">
                <span>Synthesized from top results</span>
            </div>
        </article>
    );
}
