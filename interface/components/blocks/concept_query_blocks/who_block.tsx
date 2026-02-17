import { WhoResult } from "@/stores/search_store";

export const WhoBlock = ({ data }: { data: WhoResult }) => {
    return (
        <div className="rounded-xl border-l-4 border-blue-500 bg-neutral-100 p-6 dark:bg-neutral-800">
            <h2 className="font-sans text-xl font-bold">{data.name}</h2>
            <article className="mb-4 text-sm text-neutral-500 italic">
                {data.lifespan}
            </article>
            <p className="mb-4 text-lg leading-relaxed">{data.known_for}</p>
            <div>
                <h3 className="mb-2 font-sans text-xs font-bold text-neutral-400 uppercase">
                    Key Achievements
                </h3>
                <ul className="list-disc space-y-1 pl-5">
                    {data.achievements.map((a, i) => (
                        <li key={i}>{a}</li>
                    ))}
                </ul>
            </div>
        </div>
    );
};
