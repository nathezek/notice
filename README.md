# Notice

A sophisticated, AI-powered search engine that decouples traditional web search from intelligent AI analysis. **Notice** provides immediate web results followed by insightful, background-generated AI summaries, along with interactive widgets for a seamless information retrieval experience.

![Version](https://img.shields.io/badge/version-v--0.7.0--alpha-green)

## 🛠 Technology Stack

| Hierarchy | Technology | Role |
| :--- | :--- | :--- |
| **Backend** | `Rust` / `Axum` | High-performance API server and core logic coordinator. |
| **Search Engine** | `Meilisearch` | Lightning-fast full-text search and indexing server. |
| **AI Engine** | `Google Gemini API` | Advanced LLM for content classification, summarization, and interactive tasks. |
| **Frontend** | `Next.js` / `React` | Modern, responsive interface with server-side rendering. |
| **Styling** | `Tailwind CSS 4` | Utility-first CSS for premium, high-performance UI design. |
| **Animations** | `Motion` | Smooth, interactive micro-animations for an enhanced UX. |
| **State Mgmt** | `Zustand` | Lightweight, reactive state management for the frontend. |

## ✨ Key Features

- **Decoupled Search**: Instant web results from Meilisearch with background AI processing for deep summaries.
- **AI Content Classification**: Automatically identifies and categorizes search results using Gemini.
- **Interactive Widgets**: Built-in support for calculators, currency converters, and timers powered by the backend.
- **Smart Spell-check**: SymSpell-based query correction to ensure accurate search results.
- **Rich UI Blocks**: Modular UI components for displaying universal, semantic, and specialized data.

## 🏗 Architecture

The following diagram illustrates the flow from user input to the final intelligent response:

```mermaid
graph TD
    A[User Query] --> B[Interface - Next.js]
    B --> C[Server - Rust/Axum]
    C --> D{Query Classifier}
    
    D -->|Search Query| E[Meilisearch]
    D -->|Widget Query| F[Internal Logic - Calculator/Timer/etc.]
    
    E --> G[Immediate Web Results]
    G --> B
    
    E --> H[Gemini AI Summarizer - Background]
    H --> I[AI Summary Block]
    I --> B
    
    F --> J[Interactive Widget Block]
    J --> B
```

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install)
- [Bun](https://bun.sh/) or [Node.js](https://nodejs.org/)
- [Meilisearch](https://www.meilisearch.com/docs/learn/getting_started/installation)

### Installation
1.  **Clone the repository**:
    ```bash
    git clone https://github.com/nathezek/notice.git
    cd notice
    ```

2.  **Start the Backend**:
    ```bash
    cd server
    cargo run
    ```

3.  **Start the Interface**:
    ```bash
    cd interface
    bun install
    bun run dev
    ```

---

## ✍️ Developer's Note

### Why I built this
**Notice** started as a journey to solve a personal frustration: the "information wall." Traditional search engines give you links, and LLMs give you text, but bridging the two often feels disjointed. I wanted a tool that feels like a research assistant—something that gives me the raw data immediately but works in the background to build the "bigger picture." It's about speed without sacrificing depth.

### What I've learned
This project was a deep dive into the world of **systems-level performance meets high-level AI orchestration**. 
- **Rust is a beast**: Moving from higher-level languages to Rust for the backend taught me so much about memory safety and the power of zero-cost abstractions. Handling asynchronous AI streams with Tokyo was a challenge but incredibly rewarding.
- **UX for AI is tricky**: I learned that users don't just want AI; they want to know *what* the AI is doing. This led to the decoupled search approach—showing immediate results first so the user never feels like they're just staring at a loading spinner.
- **Meilisearch is lightning**: Integrating Meilisearch was a game changer for the "instant" feel of the app.

### What's coming next
The goal is to move from "search engine" to "knowledge workspace." I want to refine how Gemini interprets complex queries and make the interactive widgets feel even more integrated into the search results.

---

## 🗺️ Future Roadmap

We're just getting started! Here are some features I'm excited to explore:

- [ ] **Personalized Knowledge Graphs**: Building a visual map of how your searches connect over time.
- [ ] **Multi-modal Insights**: Search for a topic and get a summary that combines text, generated charts, and relevant snippets.
- [ ] **Collaborative Search Sessions**: Shared workspaces where teams can research together in real-time.
- [ ] **Custom AI Personas**: Allow users to swap between different search "vibes" (e.g., "The Skeptic," "The Summarizer," "The Academic").
- [ ] **Offline-first Search Local Indexes**: Search through your own locally indexed documents with the same AI-powered speed.

---

## 🤝 Contributing
Feel free to open an issue or submit a pull request if you have ideas on how to make **Notice** even better. Let's build the future of search together!
