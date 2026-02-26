"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/lib/auth";

export default function RegisterPage() {
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const { register } = useAuth();
    const router = useRouter();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);

        if (password !== confirmPassword) {
            setError("Passwords do not match");
            return;
        }

        if (password.length < 8) {
            setError("Password must be at least 8 characters");
            return;
        }

        setLoading(true);

        try {
            await register(username, password);
            router.push("/");
        } catch (err) {
            setError(
                err instanceof Error ? err.message : "Registration failed",
            );
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="mx-auto max-w-sm px-4 pt-24">
            <h1 className="mb-6 text-center text-2xl font-bold text-white">
                Create your account
            </h1>

            <form onSubmit={handleSubmit} className="space-y-4">
                <div>
                    <label className="mb-1 block text-sm text-[var(--text-secondary)]">
                        Username
                    </label>
                    <input
                        type="text"
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2.5 text-[var(--text-primary)] transition-colors focus:border-[var(--accent)]"
                        autoFocus
                        required
                    />
                </div>

                <div>
                    <label className="mb-1 block text-sm text-[var(--text-secondary)]">
                        Password
                    </label>
                    <input
                        type="password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2.5 text-[var(--text-primary)] transition-colors focus:border-[var(--accent)]"
                        required
                        minLength={8}
                    />
                </div>

                <div>
                    <label className="mb-1 block text-sm text-[var(--text-secondary)]">
                        Confirm Password
                    </label>
                    <input
                        type="password"
                        value={confirmPassword}
                        onChange={(e) => setConfirmPassword(e.target.value)}
                        className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2.5 text-[var(--text-primary)] transition-colors focus:border-[var(--accent)]"
                        required
                    />
                </div>

                {error && (
                    <p className="text-sm text-[var(--error)]">{error}</p>
                )}

                <button
                    type="submit"
                    disabled={loading}
                    className="w-full rounded-lg bg-[var(--accent)] py-2.5 font-medium text-white transition-colors hover:bg-[var(--accent-hover)] disabled:opacity-50"
                >
                    {loading ? "Creating account..." : "Create account"}
                </button>
            </form>

            <p className="mt-6 text-center text-sm text-[var(--text-muted)]">
                Already have an account?{" "}
                <Link
                    href="/login"
                    className="text-[var(--accent)] hover:underline"
                >
                    Sign in
                </Link>
            </p>
        </div>
    );
}
