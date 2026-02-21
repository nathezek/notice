"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";

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

const formSchema = z.object({
    amount: z.number().positive(),
    from: z.string().min(1).max(10),
    to: z.string().min(1).max(10),
});

export const ConverterBlock = ({ type, data }: ConverterData) => {
    const isCurrency = type === "currency_conversion";
    const initialData = data as any; // Using any internally to share fields

    const [result, setResult] = useState(initialData.result);
    const [rate, setRate] = useState(initialData.rate || "");
    const category = initialData.category || "Currency";

    const form = useForm<z.infer<typeof formSchema>>({
        resolver: zodResolver(formSchema),
        defaultValues: {
            amount: initialData.amount,
            from: initialData.from,
            to: initialData.to,
        },
    });

    const onSubmit = async (values: z.infer<typeof formSchema>) => {
        try {
            const resp = await fetch("http://localhost:4000/search", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ query: `${values.amount} ${values.from} to ${values.to}` }),
            });
            const d = await resp.json();

            if (d.result_type === "currency_conversion" || d.result_type === "unit_conversion") {
                const parsed = JSON.parse(d.content);
                if (parsed.result) {
                    setResult(parsed.result);
                }
                if (parsed.rate) {
                    setRate(parsed.rate);
                }
            }
        } catch (e) {
            console.error("Conversion fetch failed", e);
        }
    };

    return (
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="w-full max-w-sm rounded-3xl border border-neutral-200 bg-white p-6 shadow-sm dark:border-neutral-800 dark:bg-neutral-900 border-b-4">
                <span className="text-xs font-semibold uppercase tracking-widest text-neutral-400 dark:text-neutral-500">
                    {category} Conversion
                </span>

                <form className="mt-5 flex flex-col gap-3" onSubmit={form.handleSubmit(onSubmit)}>
                    {/* Top Row: Amount & Source Unit */}
                    <div className="flex w-full items-center gap-2">
                        <input
                            type="number"
                            step="any"
                            {...form.register("amount", { valueAsNumber: true })}
                            className="flex-1 rounded-xl border border-neutral-200 bg-neutral-50 px-4 py-3 font-mono text-2xl font-medium outline-none transition-colors focus:border-blue-500 focus:bg-white dark:border-neutral-800 dark:bg-neutral-950 dark:focus:border-blue-500 dark:focus:bg-neutral-900"
                        />
                        <input
                            type="text"
                            {...form.register("from")}
                            disabled={!isCurrency}
                            className="w-24 rounded-xl border border-neutral-200 bg-neutral-50 px-3 py-3 text-center font-mono text-xl font-bold uppercase outline-none transition-colors focus:border-blue-500 focus:bg-white disabled:pointer-events-none disabled:opacity-70 dark:border-neutral-800 dark:bg-neutral-950 dark:focus:border-blue-500 dark:focus:bg-neutral-900"
                        />
                    </div>

                    <p className="text-neutral-400 pl-2 font-mono text-xl dark:text-neutral-600">=</p>

                    {/* Bottom Row: Result & Target Unit */}
                    <div className="flex w-full items-center gap-2">
                        <div className="flex-1 truncate rounded-xl bg-neutral-100 px-4 py-3 text-right font-mono text-2xl font-medium text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100">
                            {result}
                        </div>
                        <input
                            type="text"
                            {...form.register("to")}
                            disabled={!isCurrency}
                            className="w-24 rounded-xl border border-neutral-200 bg-neutral-100 px-3 py-3 text-center font-mono text-xl font-bold uppercase outline-none transition-colors focus:border-blue-500 focus:bg-white disabled:pointer-events-none disabled:bg-neutral-50 disabled:opacity-70 dark:border-neutral-800 dark:bg-neutral-800 dark:focus:border-blue-500 dark:focus:bg-neutral-900"
                        />
                    </div>
                    {/* Hidden submit button to allow Enter key to submit form */}
                    <button type="submit" className="hidden" />
                </form>

                {isCurrency && rate && (
                    <p className="mt-6 text-xs text-neutral-400 dark:text-neutral-500 text-center">
                        <span className="font-mono">
                            1 {form.getValues("from").toUpperCase()} = {rate} {form.getValues("to").toUpperCase()}
                        </span>{" "}
                        · via frankfurter.app
                    </p>
                )}
            </div>
        </div>
    );
};
