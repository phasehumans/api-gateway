# Sandboxed Code Execution Engine

execution engine for running untrusted code with strict isolation and resource control.

- API Layer: request auth, tenant isolation, validation, rate limiting, status/result APIs.
- Scheduler: bounded queue for admission control and load shedding.
- Worker Nodes: async worker pool for concurrent execution.
- Sandbox Layer: execution backends with per-run resource controls.

```text
                    +-------------------------------+
                    |           API Layer           |
                    |  Axum: Auth, Validation, RL  |
                    +-------------------------------+
                                |
                                v
                    +-------------------------------+
                    |          Scheduler            |
                    |   Bounded Queue (backpressure)|
                    +-------------------------------+
                                |
                                v
     +---------------------------------------------------------------+
     |                         Worker Pool                           |
     |   N workers pull jobs, update status, execute, persist result |
     +---------------------------------------------------------------+
                                |
                                v
                    +-------------------------------+
                    |        Sandbox Layer          |
                    |  Trait-based pluggable backend|
                    |  - Docker (default, isolated) |
                    |  - Process (unsafe fallback)  |
                    +-------------------------------+
                                |
                                v
                    +-------------------------------+
                    |  Result Store + Persistence   |
                    | in-memory + optional JSONL log|
                    +-------------------------------+
```
