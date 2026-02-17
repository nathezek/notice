import { WhereResult } from "@/stores/search_store";

export const WhereBlock = ({ data }: { data: WhereResult }) => {
    return (
        <div className="relative overflow-hidden rounded-xl bg-neutral-100 p-6 dark:bg-neutral-800">
            <h2 className="text-2xl font-bold">{data.location}</h2>
            <p className="mb-4 text-neutral-500">{data.region}</p>

            <div className="mb-4 flex flex-wrap gap-2">
                {data.facts.map((fact, i) => (
                    <span
                        key={i}
                        className="rounded-full bg-white px-3 py-1 text-sm dark:bg-neutral-700"
                    >
                        {fact}
                    </span>
                ))}
            </div>
            <div className="mt-4 rounded-lg bg-blue-500/10 p-4 text-sm text-blue-400">
                Climate/Vibe: {data.climate}
            </div>
        </div>
    );
};
