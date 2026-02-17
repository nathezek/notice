import { WhenResult } from "@/stores/search_store";

export const WhenBlock = ({ data }: { data: WhenResult }) => {
    return (
        <div className="rounded-xl border-t-4 border-purple-500 bg-neutral-100 p-6 dark:bg-neutral-800">
            <div className="mb-4 flex items-center justify-between">
                <h2 className="text-xl font-bold">{data.event}</h2>
                <span className="rounded bg-purple-500/10 px-2 py-1 font-mono text-purple-500">
                    {data.date}
                </span>
            </div>
            <p className="mb-6">{data.significance}</p>
            <div className="relative ml-2 space-y-2 border-l border-neutral-600 pl-4">
                {data.timeline.map((t, i) => (
                    <p key={i} className="text-sm text-neutral-400">
                        Before/After: {t}
                    </p>
                ))}
            </div>
        </div>
    );
};
