"use client";

import { useState } from "react";

interface MathResult {
    expression: string;
    result: string;
}

export const CalculatorBlock = ({ data }: { data: MathResult }) => {
    const [expr, setExpr] = useState(data.expression);
    const [res, setRes] = useState(data.result);
    const [resetNext, setResetNext] = useState(true);

    const evaluate = async (targetExpr: string) => {
        if (!targetExpr) return;
        try {
            const resp = await fetch("http://localhost:4000/calculate", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ expression: targetExpr }),
            });
            const d = await resp.json();
            if (d.result) {
                setRes(d.result);
            }
        } catch (e) {
            console.error("Calculate failed", e);
        }
    };

    const handlePress = async (btn: string) => {
        if (btn === "AC") {
            setExpr("");
            setRes("0");
            return;
        }

        if (btn === "=") {
            evaluate(expr);
            setResetNext(true);
            return;
        }

        let newExpr = expr;
        // If they just got a result and start typing numbers, clear the screen
        if (resetNext && /[0-9]/.test(btn)) {
            newExpr = btn;
            setResetNext(false);
        } else {
            // If they just got a result and start typing operators, append to the old expression
            newExpr = (resetNext ? res : expr) + btn;
            setResetNext(false);
        }
        setExpr(newExpr);
    };

    const buttons = [
        "(",
        ")",
        "^",
        "AC",
        "7",
        "8",
        "9",
        "/",
        "4",
        "5",
        "6",
        "*",
        "1",
        "2",
        "3",
        "-",
        "0",
        ".",
        "=",
        "+",
    ];

    return (
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="w-full max-w-[320px] rounded-3xl border border-b-4 border-neutral-200 bg-white p-4 shadow-sm dark:border-neutral-800 dark:bg-neutral-900">
                {/* Screen */}
                <div className="mb-4 flex h-24 flex-col justify-end overflow-hidden rounded-2xl bg-neutral-50 p-4 text-right dark:bg-neutral-950">
                    <p className="min-h-6 w-full truncate text-sm tracking-widest text-neutral-500 dark:text-neutral-400">
                        {expr}
                    </p>
                    <p className="w-full truncate font-mono text-4xl font-light tracking-tight text-neutral-900 dark:text-neutral-100">
                        {res}
                    </p>
                </div>

                {/* Keypad */}
                <div className="grid grid-cols-4 gap-2">
                    {buttons.map((btn) => (
                        <button
                            key={btn}
                            onClick={() => handlePress(btn)}
                            className={`flex h-14 items-center justify-center rounded-2xl text-xl transition-all active:scale-95 ${
                                btn === "="
                                    ? "bg-blue-500 font-semibold text-white shadow-sm hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700"
                                    : btn === "AC"
                                      ? "bg-red-50 font-semibold text-red-600 hover:bg-red-100 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50"
                                      : [
                                              "/",
                                              "*",
                                              "-",
                                              "+",
                                              "(",
                                              ")",
                                              "^",
                                          ].includes(btn)
                                        ? "bg-neutral-100 font-medium text-neutral-700 hover:bg-neutral-200 dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700"
                                        : "border border-neutral-100 bg-white font-medium shadow-sm hover:bg-neutral-50 dark:border-neutral-800/50 dark:bg-neutral-900 dark:hover:bg-neutral-800"
                            } `}
                        >
                            {btn}
                        </button>
                    ))}
                </div>
            </div>
        </div>
    );
};
