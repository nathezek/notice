"use client";

import React from "react";
import ReactMarkdown from "react-markdown";

interface Props {
    answer: string;
}

export default function AiAnswer({ answer }: Props) {
    return (
        <div className="ai-answer-block mb-8 rounded-xl border border-neutral-200 bg-white p-6 shadow-sm">
            <div className="mb-4 flex items-center gap-2">
                <div className="flex h-6 w-6 items-center justify-center rounded-full bg-indigo-500 text-xs text-white">
                    ✨
                </div>
                <span className="text-sm font-semibold text-neutral-800">
                    AI Overview
                </span>
            </div>

            <div className="prose prose-neutral max-w-none text-neutral-700 leading-relaxed">
                <ReactMarkdown>{answer}</ReactMarkdown>
            </div>

            <div className="mt-4 flex items-center gap-2 border-t border-neutral-100 pt-3 text-[10px] text-neutral-400">
                <span>Synthesized from top results</span>
                <span>•</span>
                <span>Generative AI can make mistakes. Check important info.</span>
            </div>
        </div>
    );
}
