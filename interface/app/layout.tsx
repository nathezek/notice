import type { Metadata } from "next";
import "./globals.css";
import ThemesProvider from "@/theme/theme_provider";
import { Navbar } from "@/modules/navbar/navbar";
import { AnimatePresence } from "motion/react";
import { Suspense } from "react";

export const metadata: Metadata = {
    title: "Notice",
    description: "New Search Engine for the web, better that google.",
};

export default function RootLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <html lang="en" suppressHydrationWarning>
            <body>
                <ThemesProvider>
                    <AnimatePresence mode="wait">
                        <Suspense fallback={null}>
                            <Navbar />
                        </Suspense>
                    </AnimatePresence>
                    {children}
                </ThemesProvider>
            </body>
        </html>
    );
}
