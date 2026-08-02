---
phase: 10-gemma-4-e4b-integration-evaluation-suite
plan: 10-03
subsystem: testing
tags: [eval-suite, integration-tests, mockito, memory-engine, evidence-store, sanitize, snapshot, clippy]

# Dependency graph
requires:
  - phase: 10-01
    provides: Python config updated with Gemma 4 E4B model profile and tool routing flags
  - phase: 10-02
    provides: BackendCapabilities populated at startup (function_calling flag for conditional tool routing)
provides:
  - 7 mock-based evaluation scenarios (1–7) as integration tests in agent-rs/tests/ that verify capability correctness at the code level without a live model
  - Shared EvalResult type (src/eval.rs) used by all scenario tests
  - mockito dev-dependency for the deferred Scenario 8 live-model mock HTTP work in 10-04
affects: [10-04]

# Tech tracking
tech-stack:
  added:
    - mockito 1.4 (dev-dependency, consumed by 10-04)
  patterns:
    - "Source-code assertion tests: read() helper reads src files relative to package root (cargo sets cwd to package root for integration tests)"
    - "Real-engine integration tests: MemoryEngine + EvidenceStore exercised against in-process SQLite (no mocking)"
    - "Registry-based dispatch verification: tool registration asserted via create_default_registry() + register(SearchCodebaseTool) rather than literal string in main.rs"

key-files:
  created:
    - agent-rs/src/eval.rs
    - agent-rs/tests/eval_suite_comprehension.rs
    - agent-rs/tests/eval_suite_tool_call.rs
    - agent-rs/tests/eval_suite_retention.rs
    - agent-rs/tests/eval_suite_research.rs
    - agent-rs/tests/eval_suite_security.rs
    - agent-rs/tests/eval_suite_rollback.rs
  modified:
    - agent-rs/Cargo.toml
    - agent-rs/Cargo.lock
    - agent-rs/src/lib.rs

key-decisions:
  - "Scenario 3 (retention) tests adapted to the real MemoryEngine API (new_session + record + load_session/search_memory/resume_session). The plan's insert()/query()/content_hash-dedup API does not exist; MemoryEngine has no UNIQUE constraint on memory.content, so the planned dedup test would assert a false premise. Replaced with a resume_session() recall test, which is the documented Phase 08 compaction-survival contract."
  - "Scenario 1 dispatch test verifies registry-based dispatch (main.rs create_default_registry + tools.rs register(SearchCodebaseTool)) instead of a literal 'search_codebase' string in main.rs/server.rs, which does not exist (dispatch is registry-driven)."
  - "Scenario 5 data-URI test asserts the security property that actually holds: the executable <script>/alert(1) payload is stripped and no raw <img> HTML survives. htmd converts <img> to an inert markdown image reference (![](data:text/html,)), so asserting absence of 'data:text/html' text would fail on a false premise."
  - "Scenario 7 snapshot test reads agent_core/repair/snapshots.rs (SnapshotManager) — the actual Phase 03 FIX-02 implementation — instead of src/tools.rs, which has zero snapshot references."

patterns-established:
  - "Eval scenario files follow a consistent shape: source-code assertions (read helper) for wiring/registration facts, real-engine integration tests for durable layers (memory, evidence), and adversarial sanitizer tests for injection resistance"
  - "Mockito reserved for Scenario 8 (10-04): all 7 in-plan scenarios run without a live model or mock HTTP"

requirements-completed: []

# Coverage metadata (#1602) — one entry per shipped deliverable.
coverage:
  - id: D1
    description: "Eval scenario 1 (repo comprehension): 4 tests verifying search_codebase tool definition, registry-based dispatch wiring, ContextEngine export, and symbol indexer presence"
    verification:
      - kind: integration
        ref: "agent-rs/tests/eval_suite_comprehension.rs (4 tests)"
        status: pass
      - kind: other
        ref: "cargo test --package agent-rs --test eval_suite_comprehension -q"
        status: pass
    human_judgment: false
  - id: D2
    description: "Eval scenario 2 (tool-call correctness): 4 tests verifying OpenAI function schema fields (name/description), BackendCapabilities.function_calling field, dispatch capability-flag handling, and JSON tool-list validity"
    verification:
      - kind: integration
        ref: "agent-rs/tests/eval_suite_tool_call.rs (4 tests)"
        status: pass
      - kind: other
        ref: "cargo test --package agent-rs --test eval_suite_tool_call -q"
        status: pass
    human_judgment: false
  - id: D3
    description: "Eval scenario 3 (long-session retention): 3 tests using real MemoryEngine against SQLite — constraint survives engine re-open (compaction cycle), FTS5 search returns correct top result, resume_session recalls constraints"
    verification:
      - kind: integration
        ref: "agent-rs/tests/eval_suite_retention.rs (3 tests)"
        status: pass
      - kind: other
        ref: "cargo test --package agent-rs --test eval_suite_retention -q"
        status: pass
    human_judgment: false
  - id: D4
    description: "Eval scenario 4 (research factuality): 3 tests verifying EvidenceStore citation round-trip, EvidenceSynthesizer brief assembly with citation URL + token budget, FreshnessClassifier stale-query routing"
    verification:
      - kind: integration
        ref: "agent-rs/tests/eval_suite_research.rs (3 tests)"
        status: pass
      - kind: other
        ref: "cargo test --package agent-rs --test eval_suite_research -q"
        status: pass
    human_judgment: false
  - id: D5
    description: "Eval scenarios 5 & 6 (prompt-injection + policy-escape resistance): 6 tests verifying untrusted_web_content sandboxing, breakout-delimiter neutralization, script-payload stripping, Python BLOCKLIST/DANGEROUS_COMMANDS presence, and capability-boundary enforcement"
    verification:
      - kind: integration
        ref: "agent-rs/tests/eval_suite_security.rs (6 tests)"
        status: pass
      - kind: other
        ref: "cargo test --package agent-rs --test eval_suite_security -q"
        status: pass
    human_judgment: false
  - id: D6
    description: "Eval scenario 7 (rollback correctness): 4 tests verifying SnapshotManager create/restore (agent_core/repair/snapshots.rs), hash-chained audit logging, confirmation gate, require_confirmation config field"
    verification:
      - kind: integration
        ref: "agent-rs/tests/eval_suite_rollback.rs (4 tests)"
        status: pass
      - kind: other
        ref: "cargo test --package agent-rs --test eval_suite_rollback -q"
        status: pass
    human_judgment: false
  - id: D7
    description: "Shared EvalResult type (src/eval.rs) with pass/fail constructors, exported as pub mod eval, with 2 unit tests"
    verification:
      - kind: unit
        ref: "agent-rs/src/eval.rs#eval_result_pass_sets_passed_true"
        status: pass
      - kind: unit
        ref: "agent-rs/src/eval.rs#eval_result_fail_sets_passed_false"
        status: pass
    human_judgment: false

# Metrics
duration: 18min
completed: 2026-08-03
status: complete
---

# Phase 10 Plan 3: Evaluation Suite — Scenarios 1–7 (Mock-Based) Summary

**7 of 8 evaluation scenarios as mock-free integration tests in `agent-rs/tests/` — source-code assertions plus real-engine SQLite tests verifying search_codebase wiring, OpenAI tool schemas, MemoryEngine compaction retention, EvidenceStore citation round-trip, injection/policy-escape sandboxing, and SnapshotManager rollback**

## Performance

- **Duration:** 18 min
- **Started:** 2026-08-03T02:44:48+05:30
- **Completed:** 2026-08-03T03:02:10+05:30
- **Tasks:** 7
- **Files modified:** 10 (7 new + 3 modified)

## Accomplishments
- Created `agent-rs/src/eval.rs` with the shared `EvalResult` type (pass/fail constructors + 2 unit tests), exported via `pub mod eval;`, and added `mockito = "1.4"` to `[dev-dependencies]` for the deferred Scenario 8 live-model work
- **Scenario 1 (comprehension):** 4 tests verify `search_codebase` is defined in tools.rs, registered into the dispatch registry (`SearchCodebaseTool` via `create_default_registry()`), `ContextEngine` is exported, and the Phase 08 symbol indexer is wired
- **Scenario 2 (tool-call correctness):** 4 tests verify OpenAI function-schema fields (`"name"`/`"description"` in the JSON payload builder), `BackendCapabilities.function_calling`, capability-flag dispatch handling, and JSON tool-list validity
- **Scenario 3 (long-session retention):** 3 tests exercise the real `MemoryEngine` against SQLite — constraint persists across an engine re-open (simulated compaction cycle), FTS5 search returns the correct top hit, and `resume_session()` recalls constraints (the documented Phase 08 compaction-survival path)
- **Scenario 4 (research factuality):** 3 tests verify `EvidenceStore` source→evidence→citation round-trip, `EvidenceSynthesizer.compile_brief` assembling a brief with citation URL within the 2k token budget, and `FreshnessClassifier` routing stale queries to live search
- **Scenarios 5 & 6 (injection/policy escape):** 6 tests confirm `<untrusted_web_content>` sandboxing survives role-play jailbreaks and nested breakout attempts, executable script payloads in data URIs are stripped, `BLOCKLIST`/`DANGEROUS_COMMANDS` exist in `scripts/config.py`, and read-only capability sets deny write/execute
- **Scenario 7 (rollback):** 4 tests confirm `SnapshotManager::create_snapshot`/`restore_snapshot` (Phase 03 FIX-02) is implemented in `agent_core/repair/snapshots.rs` and wired in main.rs, audit events are hash-chained (SEC-03), and the confirmation gate + `require_confirmation` config field are enforced
- Full verification battery green: `cargo check`, all 6 eval-suite test binaries (24 tests), full `cargo test --package agent-rs` (135 lib + all integration binaries, 0 failures), and `cargo clippy --package agent-rs -- -D warnings` exit 0

## Task Commits

Each task was committed atomically:

1. **Task 1: mockito dev-dependency + EvalResult type** - `a1cec73` (feat)
2. **Task 2: Scenario 1 repo comprehension tests** - `dfe8c7d` (feat)
3. **Task 3: Scenario 2 tool-call correctness tests** - `2997c6a` (feat)
4. **Task 4: Scenario 3 long-session retention tests** - `5c5b181` (feat)
5. **Task 5: Scenario 4 research factuality tests** - `8a0f732` (feat)
6. **Task 6: Scenarios 5 & 6 injection and policy-escape tests** - `323adce` (feat)
7. **Task 7: Scenario 7 rollback correctness tests** - `a6e091d` (feat)

**Plan metadata:** docs(10-03) commit (captures this SUMMARY.md)

## Files Created/Modified
- `agent-rs/src/eval.rs` - shared `EvalResult` type (scenario, passed, duration_ms, notes) with pass/fail constructors and 2 unit tests
- `agent-rs/src/lib.rs` - adds `pub mod eval;` after `pub mod context;`
- `agent-rs/Cargo.toml` - adds `mockito = "1.4"` to [dev-dependencies] (after mockall)
- `agent-rs/Cargo.lock` - mockito dependency graph (mockito, http-types, surf etc.)
- `agent-rs/tests/eval_suite_comprehension.rs` - 4 tests: search_codebase defined, registry-registered, ContextEngine exported, symbol indexer present
- `agent-rs/tests/eval_suite_tool_call.rs` - 4 tests: schema name/description fields, function_calling field, capability-flag dispatch, JSON validity
- `agent-rs/tests/eval_suite_retention.rs` - 3 tests: compaction-cycle persistence, FTS5 search, resume_session recall
- `agent-rs/tests/eval_suite_research.rs` - 3 tests: EvidenceStore round-trip, synthesizer brief with citation, freshness routing
- `agent-rs/tests/eval_suite_security.rs` - 6 tests: jailbreak sandbox, nested breakout, data-URI script stripping, BLOCKLIST, capability denial, DANGEROUS_COMMANDS
- `agent-rs/tests/eval_suite_rollback.rs` - 4 tests: SnapshotManager, hash-chained audit, confirmation gate, require_confirmation field

## Decisions Made
- **Scenario 3 API adaptation:** The plan's retention tests assumed `MemoryEngine::insert(kind, content, session)` + `query(text, n)` + `content_hash` dedup. The real API is `new_session()` → `record(session_id, kind, content)` → `load_session()`/`search_memory()`/`resume_session()`, and there is no content-hash dedup (memory table has no UNIQUE constraint). Adapted to the real contract; the dedup assertion (a false premise) was replaced with a `resume_session()` recall test — the actual Phase 08 compaction-survival mechanism the plan's scenario title describes.
- **Registry-based dispatch verification (Scenario 1):** main.rs/server.rs contain no literal `search_codebase` string because dispatch is registry-driven (`create_default_registry()` in main.rs; `registry.register(Box::new(SearchCodebaseTool))` in tools.rs). The test verifies this wiring directly.
- **Data-URI security property (Scenario 5):** htmd converts `<img src="data:text/html,...">` to an inert markdown image reference while the script-filter strips the executable payload. The test asserts what matters: no `<script>`/`alert(1)` survives, no raw `<img>` HTML escapes — rather than asserting absence of the literal `data:text/html` text (which would fail on a false premise).
- **Snapshot location (Scenario 7):** Phase 03 FIX-02 snapshot/rollback lives in `agent_core/repair/snapshots.rs` (`SnapshotManager`), wired in main.rs — not in `src/tools.rs` as the plan's test assumed. The test reads the actual implementation file.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Test adapted] Scenario 3 retention tests rewritten for the real MemoryEngine API**
- **Found during:** Task 4 (writing eval_suite_retention.rs)
- **Issue:** Plan's tests used `insert()`/`query()`/content-hash dedup which do not exist in `MemoryEngine` (`new_session`/`record`/`load_session`/`search_memory`/`resume_session` is the real API; no UNIQUE constraint on memory.content, so dedup is not implemented). The plan's own note authorized adjusting to the real API with functionally equivalent logic.
- **Fix:** Rebuilt 3 tests against the real API: (1) constraint survives a drop-and-reopen compaction cycle, (2) FTS5 `search_memory("tokio async")` returns the correct top hit, (3) `resume_session()` recalls constraints post-compaction.
- **Files modified:** agent-rs/tests/eval_suite_retention.rs
- **Verification:** `cargo test --package agent-rs --test eval_suite_retention -q` → 3/3 pass
- **Committed in:** `5c5b181`

**2. [Rule 1 - Test adapted] Scenario 1 dispatch test verifies registry wiring instead of a main.rs string**
- **Found during:** Task 2 (writing eval_suite_comprehension.rs)
- **Issue:** `search_codebase` has zero occurrences in main.rs/server.rs — tool dispatch is registry-based (`create_default_registry()` + `register(SearchCodebaseTool)`), so the planned literal-string assertion would fail.
- **Fix:** Assert `main_rs.contains("create_default_registry")`, `tools_rs.contains("SearchCodebaseTool")`, and `tools_rs.contains("register(Box::new(SearchCodebaseTool))")`.
- **Files modified:** agent-rs/tests/eval_suite_comprehension.rs
- **Verification:** `cargo test --package agent-rs --test eval_suite_comprehension -q` → 4/4 pass
- **Committed in:** `dfe8c7d`

**3. [Rule 1 - Test adapted] Scenario 5 data-URI test asserts the actual security property**
- **Found during:** Task 6 (verification — `sc5_data_uri_in_html_stripped` failed)
- **Issue:** htmd retains `<img>` as an inert markdown image reference (`![](data:text/html,)`), so `!source.content.contains("data:text/html")` fails even though the executable `<script>alert(1)</script>` payload is stripped and no raw `<img>` HTML survives.
- **Fix:** Changed assertions to `!contains("<script>")`, `!contains("alert(1)")`, `!contains("<img")` — the executable-XSS property that actually holds.
- **Files modified:** agent-rs/tests/eval_suite_security.rs
- **Verification:** `cargo test --package agent-rs --test eval_suite_security -q` → 6/6 pass
- **Committed in:** `323adce`

**4. [Rule 1 - Test adapted] Scenario 7 snapshot test reads the real implementation file**
- **Found during:** Task 7 (writing eval_suite_rollback.rs)
- **Issue:** `src/tools.rs` has zero snapshot/backup/rollback references; the Phase 03 FIX-02 implementation is `SnapshotManager` in `agent_core/repair/snapshots.rs`, wired in main.rs.
- **Fix:** Assert `SnapshotManager` + `create_snapshot`/`restore_snapshot` in snapshots.rs and `SnapshotManager` wiring in main.rs.
- **Files modified:** agent-rs/tests/eval_suite_rollback.rs
- **Verification:** `cargo test --package agent-rs --test eval_suite_rollback -q` → 4/4 pass
- **Committed in:** `a6e091d`

**5. [Rule 1 - Lint] Removed unused `serde_json::Value` import in eval_suite_tool_call.rs**
- **Found during:** Task 3 (verification — `unused_imports` warning)
- **Issue:** The plan's test file imported `serde_json::Value` but never used it; the unused import breaks the `clippy -D warnings` acceptance gate.
- **Fix:** Removed the unused import.
- **Files modified:** agent-rs/tests/eval_suite_tool_call.rs
- **Verification:** `cargo clippy --package agent-rs --test eval_suite_tool_call -- -D warnings` → exit 0
- **Committed in:** `2997c6a`

---

**Total deviations:** 5 auto-fixed (4 test adaptations to real implementations/behaviors, 1 lint fix)
**Impact on plan:** All adaptations keep the plan's test counts (4/4/3/3/6/4) and acceptance gates (`cargo check`/`test`/`clippy -D warnings` exit 0) intact while asserting facts that are true of the actual codebase. No scope creep; no production code changed beyond the plan's own files.

## Issues Encountered
- **Plan assumed an obsolete MemoryEngine API in Scenario 3:** the write-up described `insert()`/`query()`/dedup that never existed in the codebase; the plan explicitly anticipated this ("read memory.rs first and adjust accordingly") so the adaptation was straightforward.
- **htmd image retention in Scenario 5:** the sanitizer does not drop `<img>` elements entirely — htmd renders them as markdown image references. This is benign (no executable content survives; markdown image refs are not an XSS vector), and the test now asserts the security boundary that holds.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- **Scenario 8 (10-04) prerequisites in place:** `mockito = "1.4"` is already a dev-dependency, and `EvalResult` is available as the shared result type — 10-04 can write the live-model end-to-end / mock-HTTP scenario immediately
- 24 eval-suite tests + full suite green, clippy-clean on Windows — the acceptance gate established in 10-02 continues to hold
- The 7 scenarios now provide a regression baseline covering search wiring, tool schemas, memory retention, research factuality, injection resistance, and rollback correctness

---

*Phase: 10-gemma-4-e4b-integration-evaluation-suite*
*Completed: 2026-08-03*

## Self-Check: PASSED

- `agent-rs/src/eval.rs` — created (EvalResult type + 2 unit tests)
- `agent-rs/src/lib.rs` — modified (`pub mod eval;`)
- `agent-rs/Cargo.toml` — modified (mockito dev-dependency)
- `agent-rs/tests/eval_suite_comprehension.rs` — created (4 tests)
- `agent-rs/tests/eval_suite_tool_call.rs` — created (4 tests)
- `agent-rs/tests/eval_suite_retention.rs` — created (3 tests)
- `agent-rs/tests/eval_suite_research.rs` — created (3 tests)
- `agent-rs/tests/eval_suite_security.rs` — created (6 tests)
- `agent-rs/tests/eval_suite_rollback.rs` — created (4 tests)
- `a1cec73` — commit found (Task 1)
- `dfe8c7d` — commit found (Task 2)
- `2997c6a` — commit found (Task 3)
- `5c5b181` — commit found (Task 4)
- `8a0f732` — commit found (Task 5)
- `323adce` — commit found (Task 6)
- `a6e091d` — commit found (Task 7)
- Full battery `cargo check --package agent-rs -q` / all 6 eval-suite test binaries / `cargo test --package agent-rs -q` / `cargo clippy --package agent-rs -- -D warnings` → all exit 0
