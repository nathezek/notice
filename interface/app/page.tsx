"use client";
import { ThemeSwitcher } from "@/theme/theme_switcher";
import { IconSearch, IconX } from "@tabler/icons-react";
import { button } from "motion/react-m";
import { useState } from "react";

// The Blueprint for our structured data
interface SearchResult {
  summary: string;
  quick_facts: string[];
  deep_dive: string;
}

export default function Home() {
  const [input_query, setInputQuery] = useState("");
  const [result, setResult] = useState<SearchResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleSearch = async (userQuery: string) => {
    setIsLoading(true);
    setResult(null);

    try {
      const response = await fetch("http://localhost:4000/search", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query: userQuery }),
      });

      const data = await response.json();
      // CRITICAL: Convert the JSON string from Rust into a TypeScript object
      const parsed: SearchResult = JSON.parse(data.content);
      setResult(parsed);
    } catch (error) {
      console.error("Search failed:", error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen flex-col items-center p-12 font-sans dark:bg-neutral-900">
      <h1 className="mb-8 text-3xl font-bold">Concept Search</h1>
      <ThemeSwitcher />

      <form
        className="relative flex h-14 w-full max-w-2xl items-center justify-between gap-x-2 rounded-xl bg-neutral-200 px-4 dark:bg-neutral-800"
        onSubmit={(e) => {
          e.preventDefault();
          handleSearch(input_query);
        }}
      >
        {!input_query && (
          <IconSearch
            className="text-neutral-500 hover:text-neutral-700"
            size={20}
          />
        )}
        <input
          type="text"
          value={input_query}
          className="flex-1 bg-transparent p-2 outline-none"
          placeholder="Search..."
          onChange={(e) => setInputQuery(e.target.value)}
        />
        {input_query && (
          <button
            className="flex h-full w-10 cursor-pointer items-center justify-center opacity-85"
            type="button"
            onClick={() => setInputQuery("")}
          >
            <IconX size={18} />
          </button>
        )}
      </form>

      <div className="mt-12 w-full max-w-2xl space-y-6">
        {isLoading && (
          <p className="animate-pulse text-center text-neutral-500">
            Processing...
          </p>
        )}

        {result && (
          <article className="font-normal">
            <section className="mb-6">
              <h2 className="text-xs font-bold tracking-widest text-neutral-400 uppercase">
                Summary
              </h2>
              <p className="mt-2 text-xl font-normal text-neutral-900 dark:text-neutral-50">
                {result.summary}
              </p>
            </section>

            <section className="mb-6">
              <h2 className="text-xs font-bold tracking-widest text-neutral-400 uppercase">
                Quick Facts
              </h2>
              <ul className="mt-3 list-inside list-disc space-y-2 text-neutral-700 dark:text-neutral-300">
                {result.quick_facts.map((fact, i) => (
                  <li key={i} className="pl-2">
                    {fact}
                  </li>
                ))}
              </ul>
            </section>

            <section>
              <h2 className="text-xs font-bold tracking-widest text-neutral-400 uppercase">
                Deep Dive
              </h2>
              <p className="mt-2 leading-relaxed text-neutral-700 dark:text-neutral-300">
                {result.deep_dive}
              </p>
            </section>
          </article>
        )}
      </div>
    </div>
  );
}
