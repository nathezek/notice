# Notice Interface

A premium, high-performance search interface built with **Next.js 15**, **React 19**, and **Tailwind CSS 4**. The interface focuses on delivering a smooth, interactive experience with rich visual feedback and dynamic content loading.

## 🎨 Design System

- **Styling**: Powered by **Tailwind CSS 4**, utilizing modern CSS features and a curated color palette.
- **Animations**: Driven by **Motion** (formerly Framer Motion) for fluid transitions and micro-interactions.
- **Typography**: Uses the **Geist** font family for a sleek, modern aesthetic.

## 🧱 Key Components & Modules

| Component / Store | Location | Contribution |
| :--- | :--- | :--- |
| **Search Bar** | [`modules/search-bar`](file:///home/c0mrade/Documents/projects/work/notice/interface/modules) | The primary entry point for user queries, with real-time feedback. |
| **Blocks** | [`components/blocks`](file:///home/c0mrade/Documents/projects/work/notice/interface/components/blocks) | Modular UI units for displaying different types of results (Universal, Semantic, AI). |
| **Indexer UI** | [`components/indexer`](file:///home/c0mrade/Documents/projects/work/notice/interface/components/indexer) | Management interface for search index status and configuration. |
| **Audio Store** | [`stores/audio_store.ts`](file:///home/c0mrade/Documents/projects/work/notice/interface/stores/audio_store.ts) | Manages audio processing and speech-related states. |
| **Theme Provider** | [`theme`](file:///home/c0mrade/Documents/projects/work/notice/interface/theme) | Manages the application's visual themes (Light/Dark mode). |

## 🚀 Development

### Installation
```bash
bun install
```

### Run the development server
```bash
bun run dev
```

### Build for production
```bash
bun run build
```

---

## 🏗 Frontend Architecture

The interface follows a modular architecture, leveraging **Next.js App Router** for efficient routing and **Zustand** for lightweight, decentralized state management. The UI is designed to handle the **decoupled search flow**:
1. Displaying immediate results from the Meilisearch index.
2. Rendering loading skeletons for background AI summaries.
3. Updating the view dynamically as AI processing completes.
