"use client";

interface UnitResult {
    amount: number;
    from: string;
    to: string;
    result: string;
    category: string;
}

interface CurrencyResult {
    amount: number;
    from: string;
    to: string;
    result: string;
    rate: string;
}

type ConverterData =
    | { type: "unit_conversion"; data: UnitResult }
    | { type: "currency_conversion"; data: CurrencyResult };

export const ConverterBlock = ({ type, data }: ConverterData) => {
    const isCurrency = type === "currency_conversion";
    const d = data as UnitResult & CurrencyResult;

    return (
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="rounded-2xl border border-neutral-200 bg-neutral-50 p-6 dark:border-neutral-800 dark:bg-neutral-900">
                {/* Category badge */}
                <span className="text-xs font-semibold uppercase tracking-widest text-neutral-400 dark:text-neutral-500">
                    {isCurrency ? "Currency" : d.category} Conversion
                </span>

                {/* Main display */}
                <div className="mt-3 flex items-baseline gap-4">
                    <div className="text-center">
                        <p className="font-mono text-4xl font-medium text-neutral-900 dark:text-neutral-100">
                            {d.amount}
                        </p>
                        <p className="mt-1 font-mono text-sm font-semibold uppercase text-neutral-500">
                            {d.from}
                        </p>
                    </div>

                    <p className="text-2xl text-neutral-400 dark:text-neutral-600">=</p>

                    <div className="text-center">
                        <p className="font-mono text-4xl font-semibold text-neutral-900 dark:text-neutral-100">
                            {d.result}
                        </p>
                        <p className="mt-1 font-mono text-sm font-semibold uppercase text-neutral-500">
                            {d.to}
                        </p>
                    </div>
                </div>

                {/* Exchange rate footnote for currency */}
                {isCurrency && (
                    <p className="mt-4 text-xs text-neutral-400 dark:text-neutral-500">
                        1 {d.from} = {d.rate} {d.to} · via frankfurter.app
                    </p>
                )}
            </div>
        </div>
    );
};
