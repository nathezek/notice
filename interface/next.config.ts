import type { NextConfig } from "next";

const nextConfig: NextConfig = {
    /* config options here */
    images: {
        remotePatterns: [
            {
                protocol: "https",
                hostname: "**", // Allow any domain
            },
        ],
    },
    async rewrites() {
        return [
            {
                // Proxy API calls to the Rust backend during development
                source: "/api/:path*",
                destination: "http://localhost:8080/api/:path*",
            },
            {
                source: "/health",
                destination: "http://localhost:8080/health",
            },
        ];
    },
};

export default nextConfig;
