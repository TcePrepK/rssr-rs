# CLAUDE.md — Brochure agent guide

---

## Keeping This File Current

**Update CLAUDE.md whenever you:**

- Add, rename, or restructure a module or directory
- Change a module's sole responsibility
- Add a new persisted type or data file
- Change the testing workflow
- Establish a new non-negotiable rule
- **Change the task workflow**

**Do not let CLAUDE.md drift from the code.** If you notice a stale reference (wrong file path, removed type, changed
rule), fix it in the same commit.

---

## Task Workflow

1. **On session start**: read `TASKS.md`. Pick up the highest-priority incomplete item.
2. **Priority indicators**: tasks marked with [!] are high-priority. Always pick these before non-flagged tasks
   within the same category; otherwise take the topmost incomplete item.
3. **If anything is unclear**: stop and ask the user. Do not assume scope, layout, or behavior. Just ask directly in
   your response.
4. **On completion**: remove the item from TASKS.md and append it to `changelog/next-update.md`. The file uses
   grouped sections (`## Features`, `## UI`, `## Fixes`, `## Refactor`). Each entry must include today's date:
   `- YYYY-MM-DD: Description`. Append new entries below older ones within each section. Never commit without
   doing this first. Never read the `changelog/` files — they are an append-only archive.

**Blocked entry format** (required fields):

```
- **<Task name>** — <one-line summary>
  Question: <exact question to ask the user>
```

**Before marking any task done**: run the full test workflow (below) and confirm the requirement is met.

---

## Agent Orchestration (mandatory for every task)

After reading TASKS.md and selecting a task, **never implement it directly**. Instead:

### 1. Decompose the task

Break it into independent subtasks. For each subtask identify:

- What it does
- Which files it touches
- Whether it depends on another subtask

### 2. Assign model by complexity

| Complexity | Model            | Use for                                                                                               |
|------------|------------------|-------------------------------------------------------------------------------------------------------|
| Trivial    | `ollama (local)` | Single-function body edits, constant/string/number changes, rename within one file, add one match arm |
| Simple     | `haiku`          | Read-only research, single-file multi-function changes, UI tweaks                                     |
| Medium     | `sonnet`         | Multi-file logic changes, new handlers, state machine additions                                       |
| Complex    | `opus`           | Architectural decisions, large refactors, new persisted types + full feature                          |

#### Trivial routing criteria

**Use `ollama`** when ALL the following are true:

- The change is within a single file
- No new `fn`, `struct`, `enum`, or `impl` block is created
- The instruction does not require understanding how multiple modules fit together
- The file is not `storage.rs` or `fetch.rs`
- The change does not touch `AppState`, `AppEvent`, or the state machine

**Use `haiku` minimum** when any of the above is false, or when the instruction contains phrases like "based on", "
similar to", or "following the pattern of".

**Routing heuristic:** Can you fully specify the output without the model needing to reason about project architecture?
If yes → ollama. If not → haiku minimum.

**Dispatch syntax for trivial tasks:**

```bash
echo '{"file":"src/path/to/file.rs","instruction":"your instruction here","context_files":[]}' | python scripts/ollama_agent.py
```

Default model is `gemma4:e4b`. Override with `--model <name>` if needed. Result is JSON
`{"path": "...", "content": "..."}`. Apply with `Edit` or `Write` tool.

**Fallback:** If `ollama_agent.py` returns `{"error": "..."}`, escalate the task to `haiku`. Never silently drop the
error.

### 3. Present the dispatch plan — wait for approval

Before firing any agent, output a plan table and **stop**. Do not proceed until user approves.
Output a markdown table exactly matching this format:

```
Tasks selected: <list>

Agent plan:
| # | Task | Subtask | Model  | Rationale |
|---|------|---------|--------|-----------|
| 1 | ...  | ...     | haiku  | read-only |
| 2 | ...  | ...     | sonnet | multi-file logic |

Total: X ollama, Y haiku, Z sonnet, W opus. Proceed? (y / adjust)
```

User may reply "y", adjust individual models, drop agents, or merge subtasks. Incorporate feedback before dispatching.

### 4. Dispatch agents in parallel

Use the `Agent` tool. Independent subtasks launch simultaneously. Sequential subtasks (B depends on A's output) chain.

**One agent per file group:** Subtasks that all edit the same file must be merged into a single agent. Never split
same-file changes across multiple agents — it causes redundant file reads and potential conflicts.

**Ollama dispatch uses `Bash`, not `Agent`:** Trivial tasks routed to `ollama` must be dispatched with the `Bash` tool
running `ollama_agent.py`. The `Agent` tool always hits the Claude API regardless of any model label. Apply the returned
content with `Edit` or `Write`.

Each agent prompt must include:

- Exact file paths to read/edit
- The specific change required
- Constraints (e.g., "do not touch `ui/`", "read-only, return findings only")
- Expected return value ("return the new struct definition" / "make the change and return nothing")

Agents do **not** inherit this session's context. Give them everything they need inline.

### 5. Review and integrate

Collect agent results. Verify no conflicts. Run the test workflow yourself (agents do not run
`cargo check/test/clippy`).
Commit per the commit discipline rules.

---

## Multi-task batching

On session start, pick **multiple tasks** when sensible — not just one. Batch criteria:

- Prefer grouping tasks of similar complexity (e.g., several small UI tasks together)
- Max batch: however many can be decomposed without subtask conflicts (shared file writes)
- Present all selected tasks in the single dispatch plan table above
- One combined approval covers all selected tasks

---

## AST Cache (rust-ast-extractor)

The project is indexed with [`rust-ast-extractor`](https://github.com/TcePrepK/rust-ast-extractor). The cache lives in
`.ast-cache/` (gitignored).

**Before reading a source file**, check the cache first — it's faster and gives you signatures, docs, and line numbers
without opening the file:

```bash
# Get structured summary of a file (items, signatures, docs)
rust-ast-extractor get src/app.rs

# Get raw source of one specific item
rust-ast-extractor get src/app.rs::App
rust-ast-extractor get src/handlers/feed_list.rs::handle_feed_list_input

# Re-index after editing source files
rust-ast-extractor index src/
```

**When to use it:**

- Before asking "what does X function do?" — `get src/file.rs::fn_name` gives you the source instantly
- When planning which files to touch — `get src/file.rs` shows all items with signatures and doc comments
- After making changes — re-index so the cache stays current

**Re-index rule:** Run `rust-ast-extractor index src/` at the start of any session where you plan to edit source files,
or after any significant changes. The tool skips unchanged files, so it's fast.

---

## Module Map

```
src/
├── main.rs           Bootstrap + MPSC event loop
├── app.rs            App struct + navigation methods (next/previous/select/unselect)
├── fetch.rs          All async network I/O (feeds, images, readability)
├── storage.rs        All disk I/O (feeds.json, user_data.json, articles.json, categories.json)
├── models/
│   ├── mod.rs        Constants, Category, FeedTreeItem, FeedEditorMode; re-exports all sub-modules
│   ├── core_types.rs Feed, Article, UserData structs
│   ├── feed.rs       impl Feed + impl Article (display helpers only — no I/O)
│   ├── navigation.rs AppState, Tab, SettingsItem, AddFeedStep, FeedSource
│   └── events.rs     AppEvent (MPSC channel messages)
├── handlers/
│   ├── mod.rs        handle_key dispatch
│   ├── feed_list.rs  FeedList + FavoriteFeedList key handling
│   ├── article.rs    ArticleList/Detail, read/star toggles, open article
│   ├── settings.rs   SettingsList, AddFeed, OPML paths, ClearData
│   └── feed_editor.rs FeedEditor rename/move/delete
└── ui/
    ├── mod.rs        Catppuccin Mocha color constants + draw() entry point
    ├── chrome.rs     Tab bar + footer
    ├── content.rs    Sidebar, article list, article detail (shared by Feeds + Favorites tabs)
    ├── editor.rs     Feed editor full-screen view
    ├── settings.rs   Settings menu
    └── popups.rs     Add-feed, OPML path, confirm-delete modals
```

---

### Splitting rules

- **File-level**: if a single file contains hunks of different types, use `git add -p` to stage only the relevant hunks
  for each commit.
- **Minimum commits per session**: one per type touched. Changing a handler (fix), its rendering (ui), and CLAUDE.md (
  docs) = 3 commits.
- Never mix `feat`/`fix`/`ui`/`refactor` with `docs` in one commit.