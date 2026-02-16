import { ReactNode } from "react";

export const Article = ({ children }: { children: ReactNode }) => {
    return (
        <>
            <article className="font-extralight tracking-wider dark:text-neutral-200">
                {children}
            </article>
        </>
    );
};
