"use client";

import Link from "next/link";
import { useAuth } from "@/lib/auth";

export default function Navbar() {
    const { user, loading, logout } = useAuth();

    return (
        <nav className="border-b border-neutral-200 dark:border-neutral-700">
            <div className="mx-auto flex h-14 max-w-5xl items-center justify-between px-4">
                <Link
                    href="/"
                    className="text-lg font-bold tracking-tight text-neutral-700 transition-colors"
                >
                    Notice
                </Link>

                <div className="flex items-center gap-4">
                    {loading ? (
                        <span className="text-sm">...</span>
                    ) : user ? (
                        <>
                            <Link
                                href="/profile"
                                className="text-sm transition-colors hover:text-white"
                            >
                                {user.username}
                            </Link>
                            <button
                                onClick={logout}
                                className="text-sm transition-colors hover:text-white"
                            >
                                Sign out
                            </button>
                        </>
                    ) : (
                        <>
                            <Link
                                href="/login"
                                className="text-sm transition-colors hover:text-white"
                            >
                                Sign in
                            </Link>
                            <Link
                                href="/register"
                                className="rounded-lg px-3 py-1.5 text-sm text-white transition-colors"
                            >
                                Sign up
                            </Link>
                        </>
                    )}
                </div>
            </div>
        </nav>
    );
}
