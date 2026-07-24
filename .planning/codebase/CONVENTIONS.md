# Coding Conventions & Development Patterns

**Last Updated:** 2026-07-24

## Coding Standards & Style Guidelines

### 1. Rust (`agent-rs/`)
- **Formatting:** Formatted according to standard `rustfmt` rules.
- **Naming Conventions:**
  - Modules & Files: `snake_case` (e.g., `tool_runtime.rs`, `context_reset.rs`).
  - Structs, Enums, Traits: `PascalCase` (e.g., `SecurityPolicy`, `AgentState`).
  - Functions & Variables: `snake_case` (e.g., `execute_tool`, `validate_path`).
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_TOKEN_LIMIT`).
- **Error Handling:**
  - Avoid `unwrap()` or `expect()` in production engine code; propagate errors with `?` or handle explicitly via `match`/`if let`.
  - Use strongly typed error enums or `Result<T, Box<dyn std::error::Error + Send + Sync>>` for module boundaries.
- **Async & Concurrency:**
  - Use `tokio` channels (`mpsc`, `watch`, `broadcast`) for communication between background tasks (e.g. streaming LLM responses to TUI/Web).
  - Always guard shared mutable state using `Arc<TokioMutex<T>>` or `Arc<RwLock<T>>`.

### 2. Python (`scripts/` & `start.py`)
- **Formatting:** PEP 8 compliant style.
- **Naming Conventions:**
  - Files & Functions: `snake_case` (e.g., `onboarding_profile.py`, `clean_orphaned_servers`).
  - Classes: `PascalCase`.
  - Global Constants: `SCREAMING_SNAKE_CASE` (e.g., `PROJECT_DIR`).
- **Console Output:**
  - Standardized status markers for user readability:
    - `[i]` Info / Status updates
    - `[✓]` Success verification
    - `[!]` Warnings & Error conditions
- **Process Management:**
  - Always wrap process creation (`subprocess.Popen`) with explicit cleanup & signal handlers (`terminate()`, `kill()`) to prevent orphaned local LLM processes.

### 3. TypeScript & React (`web-ui/`)
- **Formatting:** ESLint + Prettier standards.
- **Naming Conventions:**
  - React Component Files: `PascalCase.tsx` (e.g., `App.tsx`).
  - Utility/CSS Files: `camelCase` or `kebab-case` (e.g., `main.tsx`, `index.css`).
  - TypeScript Types/Interfaces: `PascalCase` (e.g., `Message`, `ApprovalRequest`).
- **Component Design:**
  - Pure functional React components utilizing hooks (`useState`, `useEffect`, `useCallback`).
  - Utility-first styling via Tailwind CSS classes, avoiding inline styles.

## Security & Safety Conventions

1. **Path Normalization:**
   - Always run paths through canonicalization (`soft-canonicalize` or `path-security`) before reading or modifying files.
   - Guard against path traversal attempts (`..`, symlink escapes outside workspace root).

2. **Command Sanitation:**
   - Shell command execution must pass through `shell-sanitize` and `shell-words` parsing.
   - Command flags are validated against security policy levels (ReadOnly, GuidedRepair, Containerized).

3. **Audit Logging:**
   - Security decisions, tool execution attempts, and user approvals MUST be logged to audit trail via [agent-rs/src/audit.rs](file:///home/omprakash/helix-cli/agent-rs/src/audit.rs).
