import { HowResult } from "@/stores/search_store";

export const HowBlock = ({ data }: { data: HowResult }) => {
    return (
        <div className="space-y-4">
            <div className="flex justify-between text-sm text-neutral-500">
                <span>Process: {data.title}</span>
                <span>Difficulty: {data.difficulty}</span>
            </div>
            {data.steps.map((step) => (
                <div
                    key={step.step}
                    className="flex gap-4 rounded-lg bg-neutral-100 p-4 dark:bg-neutral-800"
                >
                    <span className="text-2xl font-black text-neutral-300">
                        #{step.step}
                    </span>
                    <p className="pt-1">{step.instruction}</p>
                </div>
            ))}
        </div>
    );
};
