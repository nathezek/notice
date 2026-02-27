"use client";

import { Suspense } from "react";
import SubmitUrl from "@/components/submit_url/submit_url";
import { useAuth } from "@/lib/auth";

function HomeContent() {
    const { user } = useAuth();

    return (
        <div className="mx-auto max-w-3xl px-4">
            <div className="flex min-h-[70vh] flex-col items-center justify-center">
                <div className="animate-in fade-in slide-in-from-bottom-4 mt-60 w-full max-w-xl duration-1000">
                    {user && (
                        <p className="mb-8 text-center text-xs text-neutral-500">
                            Welcome back,{" "}
                            <span className="font-semibold text-indigo-500">
                                {user.username}
                            </span>
                            . Your personal research assistant is ready.
                        </p>
                    )}
                    <SubmitUrl />
                </div>
            </div>
        </div>
    );
}

export default function HomePage() {
    return (
        <Suspense fallback={null}>
            <HomeContent />
        </Suspense>
    );
}
