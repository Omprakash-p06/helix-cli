# TESTING.md

## Testing Framework & Structure
*   **Python:** There is a minimal `tests/` directory at the root, but no formal CI runner (e.g. `pytest`) is strictly enforced as a gate yet. Much of the Python "testing" logic is embedded natively inside `setup.py`, which acts as an aggressive hardware benchmark.
*   **Hardware Benchmark / Gate:** During `setup.py`, a phantom `llama-server` process instance is momentarily started. A small completion prompt is evaluated, and the performance (tokens per second) is measured. If the speed falls below a certain threshold (e.g., 10 tok/s), the setup process warns users and suggests hardware reconfiguration. This acts as an integration test for the inference backend prior to use.
*   **Rust:** No explicit Rust unit-tests `#[test]` were easily observable in `main.rs`, but standard compilation safety checks apply. The loop relies on real-time LLM interaction which is inherently difficult to unit test without mocking the OpenAI-compatible REST server.

## Mocking & Coverage
*   **Missing:** There is currently no robust mocking of the local `llama.cpp` inference server for integration integration testing of the Rust orchestrator layer. Code coverage metrics are seemingly absent.
