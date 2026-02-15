"use client";
import { ThemeSwitcher } from "@/theme/theme_switcher";
import { IconSearch } from "@tabler/icons-react";
import { useEffect, useState } from "react";

export default function Home() {
  const [input_query, setInputQuery] = useState("");
  const [result, setResult] = useState(""); // Stores the full answer
  const [displayedText, setDisplayedText] = useState(""); // For the typing effect
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (result && displayedText.length < result.length) {
      const timeout = setTimeout(() => {
        setDisplayedText(result.slice(0, displayedText.length + 1));
      }, 20); // Adjust speed here (lower is faster)
      return () => clearTimeout(timeout);
    }
  }, [result, displayedText]);

  const handleSearch = async (userQuery: string) => {
    setIsLoading(true);
    setResult("");
    setDisplayedText("");

    try {
      const response = await fetch("http://localhost:4000/search", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query: userQuery }),
      });

      const data = await response.json();
      setResult(data.content);
    } catch (error) {
      setResult("Error: Could not connect to the server.");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen flex-col items-center justify-center font-sans">
      <h1 className="mb-4">notice</h1>
      <ThemeSwitcher />
      <form
        className="flex h-10 items-center gap-x-2"
        onSubmit={(e) => {
          e.preventDefault(); // Prevents the page from refreshing
          handleSearch(input_query);
        }}
      >
        <input
          type="text"
          value={input_query}
          className="rounded-md border border-neutral-400 p-2"
          placeholder="Search..."
          onChange={(e) => setInputQuery(e.target.value)}
        />
      </form>

      <div className="mt-12 min-h-25 w-full max-w-2xl rounded-lg border border-neutral-200 bg-neutral-50 p-6 shadow-sm">
        {isLoading && displayedText === "" ? (
          <p className="animate-pulse text-neutral-500">Consulting Gemini...</p>
        ) : (
          <p className="leading-relaxed whitespace-pre-wrap text-neutral-800">
            {displayedText}
          </p>
        )}
      </div>
    </div>
  );
}
