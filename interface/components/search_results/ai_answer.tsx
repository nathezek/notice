"use client";
import { Article } from "../ui/article";
import ReactMarkdown from "react-markdown";

interface AiAnswerData {
    title: string;
    summary: string;
    facts?: { label: string; value: string }[];
}

interface Props {
    answer: string;
}

export default function AiAnswer({ answer }: Props) {
    // Attempt to parse the answer as JSON
    let data: AiAnswerData;
    try {
        // Clean the string in case Gemini wrapped it in markdown code blocks
        const cleanJson = (str: string) => {
            const match = str.match(/```json\n([\s\S]*?)\n```/) || str.match(/```([\s\S]*?)```/);
            return match ? match[1].trim() : str.trim();
        };

        const cleaned = cleanJson(answer);
        data = JSON.parse(cleaned);
    } catch (e) {
        // Fallback if not JSON
        data = {
            title: "AI Overview",
            summary: answer,
        };
    }

    // Convert standalone bold lines to h3 for proper section styling
    const processedSummary = data.summary.replace(
        /^[\s]*(?:\*\*|__)(.*?)(?:\*\*|__)(:?)[\s]*$/gm,
        '### $1$2'
    );

    return (
        <article className="animate-in fade-in slide-in-from-bottom-4 duration-700">
            <div className="mb-4 flex items-center gap-2">
                <div className="flex h-6 w-6 items-center justify-center rounded-full bg-indigo-500 text-xs text-white">
                    ✨
                </div>
                <span className="text-sm font-semibold text-neutral-800 dark:text-neutral-200">
                    {data.title || "AI Overview"}
                </span>
            </div>

            <Article>
                <ReactMarkdown>{processedSummary}</ReactMarkdown>
            </Article>

            {data.facts && data.facts.length > 0 && (
                <div className="mt-6 grid grid-cols-1 gap-4 rounded-xl bg-neutral-50 p-5 dark:bg-neutral-800/50 md:grid-cols-2">
                    {data.facts.map((fact, i) => (
                        <div key={i} className="flex flex-col">
                            <span className="text-[10px] font-bold uppercase tracking-wider text-neutral-400">
                                {fact.label}
                            </span>
                            <span className="text-sm font-medium text-neutral-800 dark:text-neutral-100">
                                {fact.value}
                            </span>
                        </div>
                    ))}
                </div>
            )}

            <div className="mt-4 flex items-center gap-2 border-t border-neutral-100 pt-3 dark:border-neutral-800 text-[10px] text-neutral-400">
                <span>Synthesized from top results</span>
            </div>
        </article>
    );
}
