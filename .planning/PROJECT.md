# Helix Agent

## What This Is

Helix Agent is a local-first, agentic CLI stack that runs fast on consumer and low-end laptops. It provides reliable tool-calling, local model serving, and practical automation workflows without cloud lock-in.

## Core Value

A local agent that stays usable, fast, and reliable on low-end hardware while still completing real tool-driven tasks.

## Requirements

### Validated

(None yet - ship to validate)

### Active

- [ ] Local startup is simple and resilient on Windows/Linux
- [ ] Tool-calling is reliable for common file and terminal workflows
- [ ] Runtime defaults work on low-end laptops without manual tuning
- [ ] Chat UX feels normal (not debug/noise-heavy)

### Out of Scope

- Multi-user cloud orchestration - not aligned with local-first scope
- GPU-cluster distributed inference - not needed for low-end target users

## Context

- Codebase is brownfield and already combines Python setup/runtime with Rust orchestration.
- Primary backend is llama.cpp with KoboldCPP fallback.
- User priority is local speed, low friction, and agentic reliability over broad feature surface.
- Recent pain points include startup failures, noisy chat UX, preflight overhead, and tool-call incompatibilities.

## Constraints

- **Hardware**: Must run on low-end laptops - defaults must avoid heavy memory pressure.
- **Runtime**: Local-only by default - no mandatory cloud dependency for core workflows.
- **Usability**: Setup/start should avoid brittle steps (admin lockouts, long preflights by default).
- **Compatibility**: Must handle both llama.cpp and KoboldCPP endpoint behavior.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Keep local-first architecture (Python launcher + Rust orchestrator) | Fits user goal and existing investment | - Pending |
| Make heavy preflight optional by default | Faster onboarding and less frustration | - Pending |
| Prioritize low-end runtime defaults and adaptive tuning | Core product objective | - Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check - still the right priority?
3. Audit Out of Scope - reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-21 after initialization*
