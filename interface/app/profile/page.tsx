"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/lib/auth";
import { api, type KgResponse, type KgContextResponse } from "@/lib/api";

export default function ProfilePage() {
    const { user, loading: authLoading } = useAuth();
    const router = useRouter();

    const [kg, setKg] = useState<KgResponse | null>(null);
    const [context, setContext] = useState<KgContextResponse | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (authLoading) return;
        if (!user) {
            router.push("/login");
            return;
        }

        const loadData = async () => {
            try {
                const [kgData, ctxData] = await Promise.all([
                    api.getMyKg(),
                    api.getMyContext(),
                ]);
                setKg(kgData);
                setContext(ctxData);
            } catch (err) {
                setError(
                    err instanceof Error ? err.message : "Failed to load data",
                );
            } finally {
                setLoading(false);
            }
        };

        loadData();
    }, [user, authLoading, router]);

    if (authLoading || loading) {
        return (
            <div className="mx-auto max-w-3xl px-4 pt-12 text-center">
                <p className="text-(--text-muted)">Loading...</p>
            </div>
        );
    }

    if (error) {
        return (
            <div className="mx-auto max-w-3xl px-4 pt-12">
                <p className="text-(--error)">{error}</p>
            </div>
        );
    }

    if (!user || !kg) return null;

    // Find max weight for bar scaling
    const maxWeight = Math.max(...kg.entities.map((e) => e.weight), 1);

    return (
        <div className="mx-auto max-w-3xl px-4 py-8">
            {/* Header */}
            <div className="mb-8">
                <h1 className="text-2xl font-bold text-white">
                    {user.username}
                </h1>
                <p className="mt-1 text-sm text-[var(--text-muted)]">
                    Your personal knowledge graph
                </p>
            </div>

            {/* Stats */}
            <div className="mb-8 grid grid-cols-3 gap-4">
                <StatCard label="Entities" value={kg.entity_count} />
                <StatCard label="Relationships" value={kg.relationship_count} />
                <StatCard
                    label="Context Active"
                    value={context?.has_context ? "Yes" : "Not yet"}
                />
            </div>

            {/* Interest Profile */}
            <section className="mb-8">
                <h2 className="mb-4 text-lg font-semibold text-white">
                    Interest Profile
                </h2>

                {kg.entities.length === 0 ? (
                    <p className="text-sm text-[var(--text-muted)]">
                        Start searching to build your knowledge graph. Each
                        search teaches Notice about your interests.
                    </p>
                ) : (
                    <div className="space-y-2">
                        {kg.entities
                            .sort((a, b) => b.weight - a.weight)
                            .slice(0, 20)
                            .map((entity) => (
                                <div
                                    key={entity.id}
                                    className="flex items-center gap-3"
                                >
                                    <span className="w-40 truncate text-sm text-[var(--text-secondary)]">
                                        {entity.name}
                                    </span>
                                    <span className="w-20 text-xs text-[var(--text-muted)]">
                                        {entity.type}
                                    </span>
                                    <div className="h-2 flex-1 overflow-hidden rounded-full bg-[var(--bg-tertiary)]">
                                        <div
                                            className="h-full rounded-full bg-[var(--accent)] transition-all"
                                            style={{
                                                width: `${(entity.weight / maxWeight) * 100}%`,
                                            }}
                                        />
                                    </div>
                                    <span className="w-8 text-right text-xs text-[var(--text-muted)]">
                                        {entity.weight}
                                    </span>
                                </div>
                            ))}
                    </div>
                )}
            </section>

            {/* Search Context */}
            {context && context.has_context && (
                <section className="mb-8">
                    <h2 className="mb-4 text-lg font-semibold text-white">
                        Search Context
                    </h2>
                    <p className="mb-3 text-sm text-[var(--text-muted)]">
                        These terms are automatically used to personalize your
                        search results:
                    </p>
                    <div className="flex flex-wrap gap-2">
                        {context.top_interests.map((interest) => (
                            <span
                                key={interest.term}
                                className="rounded-full border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-1 text-sm text-[var(--text-secondary)]"
                            >
                                {interest.term}
                                <span className="ml-1 text-[var(--text-muted)]">
                                    ({interest.weight})
                                </span>
                            </span>
                        ))}
                    </div>
                </section>
            )}

            {/* Relationships */}
            {kg.relationships.length > 0 && (
                <section>
                    <h2 className="mb-4 text-lg font-semibold text-white">
                        Connections
                    </h2>
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                        {kg.relationships
                            .sort((a, b) => b.weight - a.weight)
                            .slice(0, 20)
                            .map((rel, i) => (
                                <div
                                    key={i}
                                    className="flex items-center gap-2 py-1 text-sm text-[var(--text-secondary)]"
                                >
                                    <span>{rel.from}</span>
                                    <span className="text-[var(--text-muted)]">
                                        →
                                    </span>
                                    <span>{rel.to}</span>
                                    <span className="ml-auto text-xs text-[var(--text-muted)]">
                                        ×{rel.weight}
                                    </span>
                                </div>
                            ))}
                    </div>
                </section>
            )}
        </div>
    );
}

function StatCard({ label, value }: { label: string; value: number | string }) {
    return (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-4 text-center">
            <div className="text-2xl font-bold text-white">{value}</div>
            <div className="mt-1 text-xs text-[var(--text-muted)]">{label}</div>
        </div>
    );
}
