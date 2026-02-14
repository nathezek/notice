"use client";
import { IconSearch } from "@tabler/icons-react";
import { useState } from "react";

export default function Home() {
  const [input_query, setInputQuery] = useState("");

  const handleSearch = async (userQuery: string) => {
    const response = await fetch("http://localhost:4000/search", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query: userQuery }),
    });

    const data = await response.json();
    console.log(data.content);
  };

  return (
    <div className="flex min-h-screen flex-col items-center justify-center font-sans">
      <h1 className="mb-4">notice</h1>
      <div className="flex h-10 items-center gap-x-2">
        <input
          type="text"
          value={input_query}
          className="rounded-md border border-neutral-400 p-2"
          placeholder="Search..."
          onChange={(e) => setInputQuery(e.target.value)}
        />
        <button className="flex h-full items-center justify-center rounded-md bg-neutral-800 p-2 text-neutral-100">
          <IconSearch size={18} onClick={() => handleSearch(input_query)} />
        </button>
      </div>
    </div>
  );
}
