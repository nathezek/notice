"use client";
import { Article } from "../ui/article";

interface Props {
    answer: string;
}

export default function AiAnswer({ answer }: Props) {
    return (
        <article>
            <div className="mb-4 flex items-center gap-2">
                <div className="flex h-6 w-6 items-center justify-center rounded-full bg-indigo-500 text-xs text-white">
                    ✨
                </div>
                <span className="text-sm font-semibold text-neutral-800">
                    AI Overview
                </span>
            </div>

            <Article>{answer}</Article>

            <div className="mt-4 flex items-center gap-2 border-t border-neutral-100 pt-3 text-[10px] text-neutral-400">
                <span>Synthesized from top results</span>
            </div>
        </article>
    );
}
