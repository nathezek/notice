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
                <h1 className="text-2xl font-bold text-neutral-700">
                    {user.username}
                </h1>
                <p className="mt-1 text-sm">Your personal knowledge graph</p>
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
                <h2 className="mb-4 text-lg font-semibold text-neutral-600">
                    Interest Profile
                </h2>

                {kg.entities.length === 0 ? (
                    <p className="text-sm">
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
                                    <span className="w-40 truncate text-sm">
                                        {entity.name}
                                    </span>
                                    <span className="w-20 text-xs">
                                        {entity.type}
                                    </span>
                                    <div className="h-2 flex-1 overflow-hidden rounded-full">
                                        <div
                                            className="h-full rounded-full transition-all"
                                            style={{
                                                width: `${(entity.weight / maxWeight) * 100}%`,
                                            }}
                                        />
                                    </div>
                                    <span className="w-8 text-right text-xs">
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
                    <h2 className="mb-4 text-lg font-semibold text-neutral-600">
                        Search Context
                    </h2>
                    <p className="mb-3 text-sm">
                        These terms are automatically used to personalize your
                        search results:
                    </p>
                    <div className="flex flex-wrap gap-2">
                        {context.top_interests.map((interest) => (
                            <span
                                key={interest.term}
                                className="rounded-full border px-3 py-1 text-sm"
                            >
                                {interest.term}
                                <span className="ml-1">
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
                    <h2 className="mb-4 text-lg font-semibold text-neutral-600">
                        Connections
                    </h2>
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                        {kg.relationships
                            .sort((a, b) => b.weight - a.weight)
                            .slice(0, 20)
                            .map((rel, i) => (
                                <div
                                    key={i}
                                    className="flex items-center gap-2 py-1 text-sm"
                                >
                                    <span>{rel.from}</span>
                                    <span className="">→</span>
                                    <span>{rel.to}</span>
                                    <span className="ml-auto text-xs">
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
        <div className="rounded-xl border p-4 text-center">
            <div className="text-2xl font-bold text-neutral-600">{value}</div>
            <div className="mt-1 text-xs">{label}</div>
        </div>
    );
}
