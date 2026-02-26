"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/lib/auth";

export default function LoginPage() {
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const { login } = useAuth();
    const router = useRouter();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);
        setLoading(true);

        try {
            await login(username, password);
            router.push("/");
        } catch (err) {
            setError(err instanceof Error ? err.message : "Login failed");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="mx-auto max-w-sm px-4 pt-24">
            <h1 className="mb-6 text-center text-2xl font-bold text-white">
                Sign in to Notice
            </h1>

            <form onSubmit={handleSubmit} className="space-y-4">
                <div>
                    <label className="mb-1 block text-sm">Username</label>
                    <input
                        type="text"
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        className="w-full rounded-lg border px-3 py-2.5 transition-colors"
                        autoFocus
                        required
                    />
                </div>

                <div>
                    <label className="mb-1 block text-sm">Password</label>
                    <input
                        type="password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        className="w-full rounded-lg border px-3 py-2.5 transition-colors"
                        required
                    />
                </div>

                {error && <p className="text-sm text-(--error)">{error}</p>}

                <button
                    type="submit"
                    disabled={loading}
                    className="w-full rounded-lg py-2.5 font-medium text-white transition-colors disabled:opacity-50"
                >
                    {loading ? "Signing in..." : "Sign in"}
                </button>
            </form>

            <p className="mt-6 text-center text-sm">
                Don&apos;t have an account?{" "}
                <Link href="/register" className="hover:underline">
                    Sign up
                </Link>
            </p>
        </div>
    );
}
