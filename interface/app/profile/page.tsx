"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth";

export default function ProfilePage() {
    const { user, loading: authLoading, logout } = useAuth();
    const router = useRouter();

    useEffect(() => {
        if (!authLoading && !user) {
            router.push("/login");
        }
    }, [user, authLoading, router]);

    if (authLoading) {
        return (
            <div className="mx-auto max-w-3xl px-4 pt-12 text-center">
                <p className="text-neutral-500">Loading...</p>
            </div>
        );
    }

    if (!user) return null;

    return (
        <div className="mx-auto max-w-3xl px-4 py-8">
            <h1 className="mb-2 text-2xl font-bold text-neutral-100">
                {user.username}
            </h1>
            <p className="mb-8 text-sm text-neutral-500">Your Notice account</p>

            <button
                onClick={() => {
                    logout();
                    router.push("/");
                }}
                className="rounded-lg border border-neutral-700 bg-neutral-800 px-4 py-2 text-sm text-neutral-300 transition-colors hover:border-neutral-500"
            >
                Sign out
            </button>
        </div>
    );
}
