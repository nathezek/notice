// ─── Types ───

export interface SearchResult {
    id: string;
    url: string;
    title: string | null;
    snippet: string;
    score: number | null;
}

export interface InstantAnswer {
    answer_type: string;
    value: string;
}

export interface SearchResponse {
    query: string;
    results: SearchResult[];
    total: number;
    instant_answer: InstantAnswer | null;
}

export interface AuthResponse {
    token: string;
    user_id: string;
    username: string;
}

export interface KgEntity {
    id: string;
    name: string;
    type: string;
    weight: number;
}

export interface KgRelationship {
    from: string;
    to: string;
    type: string;
    weight: number;
}

export interface KgResponse {
    user_id: string;
    username: string;
    entities: KgEntity[];
    relationships: KgRelationship[];
    entity_count: number;
    relationship_count: number;
}

export interface KgContextResponse {
    user_id: string;
    has_context: boolean;
    top_interests: {
        term: string;
        weight: number;
        type: string;
    }[];
}

export interface SubmitUrlResponse {
    id: string;
    url: string;
    status: string;
    message: string;
}

export interface CrawlerStatus {
    crawler: {
        pages_crawled: number;
        pages_failed: number;
        links_discovered: number;
        running: boolean;
    } | null;
    queue: {
        pending: number;
        in_progress: number;
        completed: number;
        failed: number;
    };
    meilisearch_documents: number;
}

export interface HealthResponse {
    status: string;
    service: string;
    version: string;
    dependencies: {
        postgres: string;
        meilisearch: { status: string; documents: number };
        gemini: string;
    };
}

// ─── API Client ───

class ApiClient {
    private baseUrl: string;

    constructor() {
        // In the browser, use relative URLs (Next.js rewrites handle proxying)
        // On the server, use the direct backend URL
        this.baseUrl = "";
    }

    private getToken(): string | null {
        if (typeof window === "undefined") return null;
        return localStorage.getItem("notice_token");
    }

    private headers(extra?: Record<string, string>): Record<string, string> {
        const h: Record<string, string> = {
            "Content-Type": "application/json",
            ...extra,
        };

        const token = this.getToken();
        if (token) {
            h["Authorization"] = `Bearer ${token}`;
        }

        return h;
    }

    // ── Auth ──

    async register(username: string, password: string): Promise<AuthResponse> {
        const res = await fetch(`${this.baseUrl}/api/auth/register`, {
            method: "POST",
            headers: this.headers(),
            body: JSON.stringify({ username, password }),
        });

        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.error || "Registration failed");
        }

        return res.json();
    }

    async login(username: string, password: string): Promise<AuthResponse> {
        const res = await fetch(`${this.baseUrl}/api/auth/login`, {
            method: "POST",
            headers: this.headers(),
            body: JSON.stringify({ username, password }),
        });

        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.error || "Login failed");
        }

        return res.json();
    }

    async me(): Promise<{ user_id: string; username: string } | null> {
        const token = this.getToken();
        if (!token) return null;

        try {
            const res = await fetch(`${this.baseUrl}/api/auth/me`, {
                headers: this.headers(),
            });

            if (!res.ok) return null;
            return res.json();
        } catch {
            return null;
        }
    }

    // ── Search ──

    async search(
        query: string,
        options?: { limit?: number; offset?: number; sessionId?: string },
    ): Promise<SearchResponse> {
        const params = new URLSearchParams({ q: query });
        if (options?.limit) params.set("limit", options.limit.toString());
        if (options?.offset) params.set("offset", options.offset.toString());
        if (options?.sessionId) params.set("session_id", options.sessionId);

        const res = await fetch(
            `${this.baseUrl}/api/search?${params.toString()}`,
            { headers: this.headers() },
        );

        if (!res.ok) {
            // Handle non-JSON error responses (e.g., plain text "Internal Server Error")
            const contentType = res.headers.get("content-type") || "";
            if (contentType.includes("application/json")) {
                const err = await res.json();
                throw new Error(err.error || `Search failed (${res.status})`);
            } else {
                const text = await res.text();
                throw new Error(text || `Search failed (${res.status})`);
            }
        }

        return res.json();
    }

    // ── Content ──

    async submitUrl(url: string): Promise<SubmitUrlResponse> {
        const res = await fetch(`${this.baseUrl}/api/submit`, {
            method: "POST",
            headers: this.headers(),
            body: JSON.stringify({ url }),
        });

        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.error || "Submit failed");
        }

        return res.json();
    }

    // ── Knowledge Graph ──

    async getMyKg(): Promise<KgResponse> {
        const res = await fetch(`${this.baseUrl}/api/me/kg`, {
            headers: this.headers(),
        });

        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.error || "Failed to load knowledge graph");
        }

        return res.json();
    }

    async getMyContext(): Promise<KgContextResponse> {
        const res = await fetch(`${this.baseUrl}/api/me/kg/context`, {
            headers: this.headers(),
        });

        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.error || "Failed to load context");
        }

        return res.json();
    }

    // ── Status ──

    async health(): Promise<HealthResponse> {
        const res = await fetch(`${this.baseUrl}/health`);
        return res.json();
    }

    async crawlerStatus(): Promise<CrawlerStatus> {
        const res = await fetch(`${this.baseUrl}/api/crawler/status`);
        return res.json();
    }
}

export const api = new ApiClient();
