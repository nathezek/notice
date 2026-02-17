import { WhatResult } from "@/stores/search_store";

export const WhatBlock = ({ data }: { data: WhatResult }) => {
    return (
        <div className="animate-in fade-in duration-500">
            <h2 className="mb-4 w-full border-b border-neutral-300 pb-2 text-xl font-medium tracking-tighter dark:border-neutral-700">
                {data.concept}
            </h2>
            <article className="mb-6 text-lg leading-relaxed">
                {data.definition}
            </article>
            <div className="grid grid-cols-1 gap-8 md:grid-cols-2">
                <div>
                    <span className="mb-4 block w-full border-b border-neutral-300 pb-1 font-sans text-xs font-medium tracking-tight text-neutral-400 uppercase dark:border-neutral-700">
                        Application
                    </span>
                    <article className="text-lg text-neutral-700 dark:text-neutral-300">
                        {data.application}
                    </article>
                </div>
                <div>
                    <span className="mb-4 block w-full border-b border-neutral-300 pb-1 font-sans text-xs font-medium tracking-tight text-neutral-400 uppercase dark:border-neutral-700">
                        Origin
                    </span>
                    <article className="text-lg text-neutral-700 dark:text-neutral-300">
                        {data.origin}
                    </article>
                </div>
            </div>
        </div>
    );
};
