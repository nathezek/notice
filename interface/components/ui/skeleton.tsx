"use client";

export const Skeleton = ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => {
    return (
        <div
            className={`animate-pulse rounded-md bg-neutral-200/50 dark:bg-neutral-800/50 ${className}`}
            {...props}
        />
    );
}

export const SearchResultSkeleton = () => {
    return (
        <div className="w-full space-y-6">
            {/* Header Skeleton */}
            <div className="space-y-2">
                <Skeleton className="h-8 w-3/4" />
                <Skeleton className="h-4 w-1/2" />
            </div>

            {/* Article/Body Skeleton */}
            <div className="space-y-3">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-5/6" />
                <Skeleton className="h-4 w-full" />
            </div>

            {/* Facts/Grid Skeleton */}
            <div className="grid grid-cols-1 gap-8 pt-4 md:grid-cols-2">
                <div className="space-y-2">
                    <Skeleton className="h-4 w-24" />
                    <Skeleton className="h-20 w-full" />
                </div>
                <div className="space-y-2">
                    <Skeleton className="h-4 w-24" />
                    <Skeleton className="h-20 w-full" />
                </div>
            </div>
        </div>
    );
};
