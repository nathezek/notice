"use client";

interface MathResult {
    expression: string;
    result: string;
}

export const CalculatorBlock = ({ data }: { data: MathResult }) => {
    return (
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="rounded-2xl border border-neutral-200 bg-neutral-50 p-6 dark:border-neutral-800 dark:bg-neutral-900">
                {/* Expression */}
                <p className="font-mono text-sm text-neutral-500 dark:text-neutral-400">
                    {data.expression} =
                </p>

                {/* Result */}
                <p className="mt-1 font-mono text-5xl font-semibold tracking-tight text-neutral-900 dark:text-neutral-100">
                    {data.result}
                </p>
            </div>
        </div>
    );
};
