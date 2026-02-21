import { ReactNode } from "react";

export const Article = ({ children }: { children: ReactNode }) => {
    return (
        <article className="prose prose-neutral max-w-none dark:prose-invert 
            prose-headings:font-medium prose-headings:tracking-tight 
            prose-headings:border-b prose-headings:border-neutral-200/50 dark:prose-headings:border-neutral-800/50 prose-headings:pb-2
            prose-p:leading-relaxed prose-p:text-neutral-700 dark:prose-p:text-neutral-300 prose-p:mb-6
            prose-strong:font-medium prose-strong:bg-amber-500/10 prose-strong:px-1 prose-strong:rounded-sm prose-strong:text-neutral-900 dark:prose-strong:text-neutral-100
            prose-li:marker:text-neutral-400 text-lg transition-colors duration-300">
            {children}
        </article>
    );
};
