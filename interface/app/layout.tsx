import type { Metadata } from "next";
import "./globals.css";
import ThemesProvider from "@/theme/theme_provider";
import Navbar from "@/modules/navbar/navbar";
import { AuthProvider } from "@/lib/auth";

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
                <AuthProvider>
                    <ThemesProvider>
                        <Navbar />
                        {children}
                    </ThemesProvider>
                </AuthProvider>
            </body>
        </html>
    );
}
