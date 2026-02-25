# Notice Server: The Developer's Deep-Dive Guide 🚀

Hey there! Welcome to the engine room of **Notice**. I'm the senior developer who's been living in this codebase, and I'm thrilled to show you around. This isn't just a README; this is a genuine conversation between us. We're going to peel back the layers of the `server` directory so you don't just know *what* the code does, but *why* we built it this way.

---

## 🌎 1. The Bigger Picture: How Notice Thinks

At its heart, the Notice server is a **Coordinator**. It’s not just a traditional API that fetches data from a database; it’s an intelligent gateway that decides how to fulfill a user's intent.

### The Problem We Solved
When a user types something into a search bar, they could want *anything*. 
- "What is 2+2?" (Math)
- "Price of BTC in USD" (Currency)
- "Define photosynthesis" (General Knowledge)

In most legacy systems, you'd throw everything at a search index and hope for the best. **We don't do that.**

### The "Notice" Architecture
We built a multi-layered response system:
1.  **Direct Intent (Classifiers):** Before searching the web, we use RegEx and logic to see if we can solve the problem instantly (Math, Units, Timers). This is fast and cheap.
2.  **Identity Matching (The Cache):** We hash every search query. If we've answered it before and stored it in Meilisearch, we return it instantly. This makes common searches lightning-fast.
3.  **The Live Web & AI (The Fallback):** If it's a new, general query, we scrape the live web, gather context, and let **Gemini** synthesize a perfect answer.

### Why Rust?
You'll notice we're using **Rust** with the **Axum** framework.
- **Speed:** Scraping and AI processing are heavy. Rust ensures the "glue" code doesn't add any overhead.
- **Safety:** When handling raw HTML from the web, memory safety is paramount to prevent crashes or exploits.
- **Concurrency:** Using `tokio`, we can scrape multiple websites and talk to AI APIs at the same time without blocking the main server.

---

## 📂 2. File-by-File Line-by-Line Breakdown

Let's dive into the `src` directory. I'll take you through each file, one by one.

---

### 📄 `main.rs`
**Location:** `server/src/main.rs`
**Why we need it:** This is the heart of the machine—the entry point where the server is born and the routes are defined.

#### What it currently does:
- Initializes environment variables (`.env`).
- Sets up database connections (PostgreSQL & Meilisearch).
- Defines the API endpoints (Routes).
- Orchestrates the primary "Search" logic flow.

#### How it functions:

##### `struct AppState`
- **Purpose:** To hold global data that every request handler might need.
- **Lines 43-47:**
  - `api_key`: The Gemini key.
  - `db_pool`: Our connection to PostgreSQL.
  - `meili_client`: Our connection to Meilisearch.

##### `async fn main()`
- **Purpose:** Starts the server and wires everything together.
- **Lines 67-87:** Loads config and initializes the `AppState`.
- **Lines 98-104:** Defines our **Router**. This is where we say "If a request comes to `/search/web`, send it to `handle_search_web`."

##### `async fn handle_search_web(...)`
- **Purpose:** This is the most important function in the project. It handles the initial search request.
- **Step-by-Step Breakdown:**
  - **Lines 119-126:** Sanity check. If the query is empty, we stop early. No point in wasting resources.
  - **Line 128:** We call `classifier::classify(query)`. This is the "Brain" deciding what kind of query this is.
  - **Lines 129-203 (The Specific Handlers):** If the brain says it's "Math", we try the `calculator`. If "Currency", the `currency` module. 
    - *Reason:* We prioritize these because they are deterministic and instant.
  - **Lines 204-267 (The General Logic):**
    - **Identity Check (Lines 207-226):** We hash the query and check Meilisearch. If we find an exact match, we return it. This is our "Search Cache."
    - **Fuzzy Check (Lines 228-251):** If no exact match, we search the index for *similar* titles.
    - **Live Search (Lines 252-266):** If all else fails, we call `web::search` to find real URLs from the internet.

##### `async fn fallback_to_gemini(...)`
- **Purpose:** When we can't find a stored answer, we ask the AI.
- **Line 336 (`tokio::spawn`):** Notice this? We return the answer to the user *immediately*, but in the background, we start a task to save that answer to our database.
  - *Reason:* The user shouldn't wait for our database write to finish before they see their answer.

---

### 📄 `web.rs`
**Location:** `server/src/web.rs`
**Why we need it:** This is our "eyes" on the internet. Since Notice isn't just a static database, we need a way to go out and find fresh information.

#### What it currently does:
- Scrapes search engines (Mojeek, Google, DuckDuckGo).
- Scrapes specific web pages and cleans the HTML.
- Extracts titles and primary text content for the AI to read.

#### How it functions:

##### `async fn mojeek_search(...)`
- **Purpose:** Searches the Mojeek search engine.
- **Lines 11-54:**
  - *Reasoning:* We use Mojeek first because it's "scraper-friendly." It doesn't use complex JavaScript, meaning we can get results fast with simple HTML parsing.
  - **Lines 31-50:** We use the `scraper` crate with CSS selectors like `a.ob` to find the actual links in the results.

##### `async fn google_search(...)`
- **Purpose:** Searches Google as a fallback.
- **Lines 58-99:**
  - *Strategy:* We use `gbv=1` in the URL (Line 67). This tells Google to give us the basic, old-school HTML version instead of the modern JS-heavy version which is much harder to scrape.

##### `async fn scrape(...)`
- **Purpose:** Reads the content of a single website.
- **Lines 155-224:**
  - **Lines 185-186:** Grabs the `<title>` tag so the AI knows what page it's looking at.
  - **Lines 191-203:** This is the "Cleaner." We only grab tags like `<p>`, `<article>`, and `<li>`.
  - *Rationale:* We ignore `<nav>` and `<footer>` because they are full of "noise" (links like 'Contact Us') that would confuse the AI and waste tokens.

---

### 📄 `classifier.rs`
**Location:** `server/src/classifier.rs`
**Why we need it:** To determine the **Intent** of the user. This file prevents us from sending "What is 1+1" to a search engine when a simple calculator can solve it.

#### What it currently does:
- Uses Regular Expressions (RegEx) to spot patterns like math, unit conversions, or timers.
- Labels the query so `main.rs` knows which module to call.

#### How it functions:

##### `fn math_re()` / `fn unit_re()` etc.
- **Purpose:** Compiled RegEx patterns.
- **Line 9 / 26 / 50 (`OnceLock`):**
  - *Senior Tip:* We use `OnceLock`. This ensures the RegEx is "compiled" into memory exactly once. If we did this on every request, the server would slow down significantly.

##### `pub fn classify(query: &str) -> QueryType`
- **Purpose:** The actual decision-maker.
- **Lines 73-92:** It checks the query against each pattern in order of importance. If no pattern matches, it defaults to `QueryType::General` (Live Search).

---

### 📄 `gemini.rs`
**Location:** `server/src/gemini.rs`
**Why we need it:** To turn messy web scrapes into a clean, human-readable answer. This is the "Brain" that does the actual thinking.

#### What it currently does:
- Communicates with Google's Gemini API.
- Sends a complex internal prompt to ensure the AI returns data in a specific JSON format.
- Handles "Rate Limits" (when the API tells us we're asking too fast).

#### How it functions:

##### `async fn ask_gemini(...)`
- **Purpose:** The main interaction point for search.
- **Lines 44-84 (The Prompt):** We tell the AI *exactly* how to behave. Note the instruction to "EXTENSIVELY use **bold text**." This is why your search results look so premium.
- **Lines 98-158 (The Retry Loop):**
  - *Reasoning:* Sometimes the AI API is busy (Error 429). Instead of just giving up and showing the user an error, we wait a few seconds and try again (Exponential Backoff). This makes the app feel much more stable.

##### `async fn summarize_page(...)`
- **Purpose:** Specifically used when a user "Adds a URL to the index."
- **Lines 174-247:** It takes a long webpage and shrinks it down to a 4-sentence summary. This summary is what gets saved in our database for future searches.

---

### 📄 `indexer.rs`
**Location:** `server/src/indexer.rs`
**Why we need it:** Notice's super-power is **Identity Memory**. This file manages our connection to **Meilisearch**, which allows us to find previously answered questions in milliseconds.

#### What it currently does:
- Initializes the Meilisearch client.
- Adds new "Search Concepts" to the index.
- Performs fuzzy-search lookups when the user types.

#### How it functions:

##### `pub async fn init_indexer(...)`
- **Purpose:** Startup configuration.
- **Lines 13-20:** Connects to the Meilisearch server and ensures the "pages" index exists.
  - *Design Choice:* We use `pages` as our index name. This is where both scraped websites and synthesized AI answers live.

##### `pub async fn index_page(...)`
- **Purpose:** Saves a search result.
- **Lines 22-32:** Takes our `IndexDocument` and pushes it into Meilisearch.

##### `pub async fn search_index(...)`
- **Purpose:** Finds relevant stored answers.
- **Lines 34-49:**
  - **Line 43:** `.with_limit(3)` — We only take the top 3 best matches.
  - *Rationale:* Taking more than 3 would clutter the UI and likely include irrelevant results.

---

### 📄 `db.rs`
**Location:** `server/src/db.rs`
**Why we need it:** While Meilisearch is for *searching*, **PostgreSQL** (via SQLx) is for **Storage**. This is our permanent vault for everything Notice has ever "read."

#### What it currently does:
- Manages the PostgreSQL connection pool.
- Creates the `pages` table if it’s missing.
- Saves raw scraped data and AI summaries.

#### How it functions:

##### `pub async fn init_db(...)`
- **Purpose:** Connects to Postgres and runs "Migrations."
- **Lines 17-40:** Note the `CREATE TABLE IF NOT EXISTS` (Lines 26-34).
  - *Philosophy:* The server is self-healing. If you start it on a fresh database, it creates its own structure automatically.

##### `pub async fn insert_page(...)`
- **Purpose:** The "Upsert" (Update or Insert) logic.
- **Lines 42-66:**
  - **Line 48 (`ON CONFLICT`):** This says "If we already have this URL in the database, don't throw an error—just update the content with the latest version."

---

### 📄 `calculator.rs`
**Location:** `server/src/calculator.rs`
**Why we need it:** To handle the interactive math and unit conversion features.

#### What it currently does:
- Translates "Three plus five" into `3 + 5`.
- Evaluates mathematical expressions using the `meval` crate.
- Handles complex unit conversions (length, mass, temperature, speed).

#### How it functions:

##### `pub fn normalize_math(...)`
- **Purpose:** The "Translator."
- **Lines 30-96:**
  - **Line 40 (`words_to_digits`):** Handles "nine" -> "9".
  - **Lines 43-69 (Function Aliases):** Handles "square root of 16" -> "sqrt(16)".
  - *Innovation:* This allows our search engine to understand "human math" instead of requiring users to type like programmers.

##### `pub fn eval_math(...)`
- **Purpose:** The "Executioner."
- **Lines 98-111:** Takes the translated string and computes the answer.
  - **Line 106:** Note the `.trim_end_matches('0').trim_end_matches('.')`.
  - *UX Detail:* We want results to look clean. We show `5` instead of `5.000000`.

##### `pub fn convert_unit(...)`
- **Purpose:** Orchestrates unit changes.
- **Lines 132-199:**
  - **Line 182 (Temperature):** Notice temperature has its own logic. This is because you can't just "multiply" Celsius to get Fahrenheit—there's an offset (+32). This is why a simple multiplier math table wasn't enough.

### 📄 `spell.rs`
**Location:** `server/src/spell.rs`
**Why we need it:** Users type fast and make mistakes. This file uses the **SymSpell** algorithm to suggest corrections, ensuring that even if you type "fasebook," we know you meant "Facebook."

#### What it currently does:
- Loads an English frequency dictionary at compile-time.
- Cleans and prepares queries for spell-checking.
- Suggests corrections while preserving capitalization and punctuation.

#### How it functions:

##### `fn spellchecker()`
- **Purpose:** Initializes the dictionary.
- **Lines 5-17:**
  - *Senior Tip:* Note Line 5: `include_str!`. We embed the 2MB dictionary into the server binary itself. This means we don't have to worry about missing files at runtime, and loading the dictionary is incredibly fast.

##### `pub fn correct_query(...)`
- **Purpose:** The entry point for correction.
- **Lines 20-79:**
  - **Lines 25-31:** We skip checking for very short queries or those containing numbers/quotes. This prevents "over-correcting" things that might be intentional.
  - **Line 52 (`lookup_compound`):** Unlike basic spell-checkers, `lookup_compound` can correct entire multi-word phrases at once.

---

### 📄 `currency.rs`
**Location:** `server/src/currency.rs`
**Why we need it:** To provide real-time currency exchange rates.

#### What it currently does:
- Parses queries like "100 EUR to USD".
- Fetches live market data from the **Frankfurter API**.
- Calculates and returns the converted amount.

#### How it functions:

##### `pub async fn convert_currency(...)`
- **Purpose:** Orchestrates the conversion.
- **Lines 28-69:**
  - **Line 30:** Uses RegEx to extract the amount, the "from" currency, and the "to" currency.
  - **Line 36-39:** Dynamically builds a request URL for the Frankfurter API.
  - **Line 42 (`reqwest`):** Makes the network call. Note the `.await`. This tells the server "Go do other work while we wait for the internet to respond."

---

## 🚀 3. Final Words for the New Developer

You've now seen the entire `server` directory. Some key takeaways for your first day:
1.  **Don't block the thread:** Always use `.await` for database or network calls.
2.  **Think of the User:** Notice is about speed and clarity. If you're adding a feature, ask yourself: *Can this be deterministic (fast) or does it need AI (slower)?*
3.  **Logs are your friend:** We use the `tracing` crate. If something goes wrong, check the terminal—the server will usually tell you exactly what it's thinking.

Good luck, and happy coding! You're now a senior contributor to Notice. 🎓
