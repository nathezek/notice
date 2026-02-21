"use client";

import { useEffect, useState, useRef } from "react";
import { IconPlayerPause, IconPlayerPlay, IconRotateClockwise2 } from "@tabler/icons-react";

interface TimerResult {
    seconds: number;
    query: string;
}

export const TimerBlock = ({ data }: { data: TimerResult }) => {
    const totalSeconds = data.seconds;

    const [timeLeft, setTimeLeft] = useState(totalSeconds);
    const [isRunning, setIsRunning] = useState(true);
    const [hasFinished, setHasFinished] = useState(false);

    // Store an audio object ref to play the chime
    const audioRef = useRef<HTMLAudioElement | null>(null);

    useEffect(() => {
        // Simple ascending chime using base64 audio to avoid needing static file assets
        audioRef.current = new Audio("data:audio/mp3;base64,//NExAAAAANIAAAAAExBTUUzLjEwMKqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq");
    }, []);

    useEffect(() => {
        let interval: NodeJS.Timeout;

        if (isRunning && timeLeft > 0) {
            interval = setInterval(() => {
                setTimeLeft((prev) => {
                    if (prev <= 1) {
                        setIsRunning(false);
                        setHasFinished(true);
                        audioRef.current?.play().catch(e => console.error("Audio play failed", e));
                        return 0;
                    }
                    return prev - 1;
                });
            }, 1000);
        }

        return () => clearInterval(interval);
    }, [isRunning, timeLeft]);

    const formatTime = (secs: number) => {
        const h = Math.floor(secs / 3600);
        const m = Math.floor((secs % 3600) / 60);
        const s = Math.floor(secs % 60);

        if (h > 0) {
            return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
        }
        return `${m}:${s.toString().padStart(2, "0")}`;
    };

    const handleReset = () => {
        setIsRunning(true);
        setHasFinished(false);
        setTimeLeft(totalSeconds);
        audioRef.current?.pause();
        if (audioRef.current) audioRef.current.currentTime = 0;
    };

    const handleToggle = () => {
        if (hasFinished) {
            handleReset();
        } else {
            setIsRunning(!isRunning);
        }
    };

    // Calculate circumference for SVG circle
    const radius = 90;
    const circumference = 2 * Math.PI * radius;
    // Calculate how much to offset the dash array
    const strokeDashoffset = circumference - (timeLeft / totalSeconds) * circumference;

    return (
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500 flex w-full justify-center lg:justify-start">
            <div className="flex w-full max-w-sm flex-col items-center justify-center rounded-3xl border border-neutral-200 bg-white p-8 shadow-sm border-b-4 dark:border-neutral-800 dark:bg-neutral-900">
                <span className="text-sm font-medium tracking-wide text-neutral-500 dark:text-neutral-400 mb-6 text-center line-clamp-1">
                    {data.query}
                </span>

                <div className="relative flex items-center justify-center mb-8">
                    {/* SVG Circle Timer */}
                    <svg className="w-56 h-56 -rotate-90 transform">
                        <circle
                            cx="112"
                            cy="112"
                            r={radius}
                            className="stroke-neutral-100 dark:stroke-neutral-800"
                            strokeWidth="12"
                            fill="transparent"
                        />
                        <circle
                            cx="112"
                            cy="112"
                            r={radius}
                            className={`transition-all duration-1000 ease-linear ${hasFinished ? "stroke-red-500" : "stroke-blue-500"}`}
                            strokeWidth="12"
                            fill="transparent"
                            strokeLinecap="round"
                            style={{
                                strokeDasharray: circumference,
                                strokeDashoffset: strokeDashoffset,
                            }}
                        />
                    </svg>

                    {/* Time Text Overlay */}
                    <div className="absolute inset-0 flex items-center justify-center">
                        <span className={`font-mono text-5xl font-light tracking-tight ${hasFinished ? "text-red-500 animate-pulse" : "text-neutral-900 dark:text-white"}`}>
                            {formatTime(timeLeft)}
                        </span>
                    </div>
                </div>

                <div className="flex items-center gap-4">
                    <button
                        onClick={handleToggle}
                        className={`flex h-14 w-14 items-center gap-2 rounded-full font-semibold transition-all hover:scale-105 active:scale-95 justify-center text-white shadow-sm border ${isRunning
                                ? "bg-blue-100 text-blue-600 border-blue-200 dark:bg-blue-900/30 dark:border-blue-800/50 dark:text-blue-400"
                                : "bg-blue-500 text-white border-blue-600 dark:bg-blue-600 dark:border-blue-700"}`}
                    >
                        {isRunning ? <IconPlayerPause size={24} /> : <IconPlayerPlay size={24} fill="currentColor" />}
                    </button>
                    <button
                        onClick={handleReset}
                        className="flex h-12 w-12 items-center justify-center rounded-full bg-neutral-100 text-neutral-600 transition-all hover:bg-neutral-200 active:scale-95 dark:bg-neutral-800 dark:text-neutral-400 dark:hover:bg-neutral-700"
                        title="Reset Timer"
                    >
                        <IconRotateClockwise2 size={20} />
                    </button>
                </div>
            </div>
        </div>
    );
};
