import type { InstantAnswer as InstantAnswerType } from "@/lib/api";

interface Props {
    answer: InstantAnswerType;
}

const typeLabels: Record<string, { icon: string; label: string }> = {
    calculation: { icon: "🔢", label: "Calculator" },
    definition: { icon: "📖", label: "Definition" },
    timer: { icon: "⏱️", label: "Timer" },
};

export default function InstantAnswer({ answer }: Props) {
    const meta = typeLabels[answer.answer_type] || {
        icon: "⚡",
        label: "Instant Answer",
    };

    return (
        <div className="instant-answer mb-6">
            <div className="mb-2 flex items-center gap-2">
                <span className="text-lg">{meta.icon}</span>
                <span className="text-xs font-medium tracking-wider uppercase">
                    {meta.label}
                </span>
            </div>
            <div className="text-2xl font-semibold text-white">
                {answer.value}
            </div>
        </div>
    );
}
