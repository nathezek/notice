import { WhoResult } from "@/stores/search_store";

export const WhoBlock = ({ data }: { data: WhoResult }) => {
    return (
        <div>
            <h2 className="text-xl font-bold">{data.name}</h2>
            <p className="mb-4 text-sm text-neutral-500">{data.lifespan}</p>
            <article className="mb-4 text-lg leading-relaxed tracking-wide dark:text-neutral-200">
                {data.known_for}
            </article>
            <div>
                <h3 className="mb-2 text-sm opacity-70">Key Achievements</h3>
                <ul className="serif-font list-disc space-y-1 pl-5 text-xl font-normal tracking-wide dark:text-neutral-200">
                    {data.achievements.map((a, i) => (
                        <li key={i}>{a}</li>
                    ))}
                </ul>
            </div>
        </div>
    );
};
