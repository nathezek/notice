import { WhoResult } from "@/stores/search_store";

export const WhoBlock = ({ data }: { data: WhoResult }) => {
    return (
        <div>
            <h2 className="serif-font text-xl font-bold">{data.name}</h2>
            <article className="mb-4 text-sm text-neutral-500">
                {data.lifespan}
            </article>
            <article className="mb-4 text-lg leading-relaxed">
                {data.known_for}
            </article>
            <div>
                <h3 className="mb-2 text-xs font-bold text-neutral-400">
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
