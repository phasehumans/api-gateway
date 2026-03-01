# Sandboxed Code Execution Engine (Rust)

A multi-tenant, API-driven execution engine for running untrusted code with strict isolation and resource control.

## System Architecture Diagram (Text)

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

### Separation of Concerns

- API Layer: request auth, tenant isolation, validation, rate limiting, status/result APIs.
- Scheduler: bounded queue for admission control and load shedding.
- Worker Nodes: async worker pool for concurrent execution.
- Sandbox Layer: execution backends with per-run resource controls.

## Folder Structure

```text
src/
  main.rs
  engine/
    mod.rs                 # bootstrap: config, tracing, router, workers
    api.rs                 # REST handlers
    config.rs              # environment config
    error.rs               # API error model
    metrics.rs             # counters + queue depth rendering
    models.rs              # request/response/domain models
    queue.rs               # scheduler + job queue types
    rate_limit.rs          # per-tenant token bucket
    store.rs               # in-memory records + optional persistence
    worker.rs              # worker execution lifecycle
    sandbox/
      mod.rs               # sandbox trait + backend factory
      language.rs          # language runtime mappings
      docker.rs            # secure default backend
      process.rs           # unsafe fallback backend
```

## Core Modules Definition

- `EngineConfig`: API bind, worker/queue sizing, limits, sandbox backend, tenant keys, rate limits.
- `ExecutionStore`: execution record lifecycle (`queued -> running -> final`) and replay events.
- `Scheduler`: queue admission and backpressure (`503` when full).
- `TenantRateLimiter`: per-tenant token bucket.
- `SandboxBackend` trait:
  - `DockerSandbox`: host-isolated execution with strict flags.
  - `ProcessSandbox`: local fallback for development only.
- `worker`: dispatches jobs, executes batches, classifies status, persists results.

## REST API

- `POST /v1/executions`
  - Submit code for execution.
  - Requires `x-api-key`.
- `GET /v1/executions/{id}`
  - Retrieve status metadata.
- `GET /v1/executions/{id}/result`
  - Retrieve full output (`stdout`, `stderr`, `exit_code`, `duration_ms`, replay events).
- `GET /healthz`
  - Health probe.
- `GET /metrics`
  - Prometheus-style counters/gauges.

## Execution Lifecycle Flow

1. API receives execution request.
2. Auth validates API key and maps to tenant.
3. Input validation checks language/code/args/stdin/limits/test batch.
4. Rate limiter checks tenant token bucket.
5. Scheduler enqueues job into bounded queue.
6. Worker claims job and marks status `running`.
7. Sandbox backend executes with resource constraints.
8. Output captured and truncated by `max_output_bytes`.
9. Final status persisted (`succeeded|failed|timed_out`).
10. Optional JSONL persistence appends execution record.
11. Temporary execution artifacts are cleaned deterministically.

## Security Model

- No host filesystem escape:
  - Docker backend mounts only per-job source directory read-only.
- No privilege escalation:
  - `--cap-drop ALL`, `--security-opt no-new-privileges`, read-only root FS.
- No persistent container state:
  - `docker run --rm` + per-job ephemeral working directory.
- Network disabled by default:
  - `--network none` unless tenant allowlisted and request opts in.
- Fork bomb defense:
  - `--pids-limit` + `--ulimit nproc`.
- Infinite loop defense:
  - hard execution timeout with kill path.
- Memory exhaustion defense:
  - Docker memory limit (`--memory`).
- Output explosion defense:
  - bounded output capture.

## Resource Controls

Per execution:

- CPU cores
- Memory MB
- Timeout ms
- Max process count
- Max source file size
- Max output bytes

## Scaling Strategy

- Stateless API nodes:
  - Scale horizontally behind a load balancer.
- Worker pool scaling:
  - Increase worker replicas and queue consumers.
- Queue backpressure:
  - Bounded channel prevents overload and enables load shedding.
- Multi-tenant safety:
  - API key to tenant mapping + per-tenant rate limiting + tenant-scoped result access.
- Pluggable sandbox backend:
  - Current backends: Docker, Process.
  - Designed for additional backends (microVM/WASM) by implementing `SandboxBackend`.

## Failure Handling Strategy

- Queue full: reject with `503` and remove staged record.
- Sandbox spawn failure: mark execution as failed with error.
- Timeout: kill execution, mark as timed out.
- Worker failure during run: persist failure state for the job.
- Persistence write failure: non-fatal; in-memory record still retained.
- Replay visibility: each job stores ordered execution events.

## Stretch Features Included

- Multi-language support:
  - Python, JavaScript, Rust, C.
- Compile + run:
  - Rust/C compile before execution.
- Basic compilation cache:
  - Process backend caches compiled binaries by source hash.
- AI-agent optimized mode:
  - `agent_optimized` mode increases timeout/output ceilings.
- Batch test case execution:
  - one submission can run multiple stdin test cases.
- Replay logs:
  - execution event trail in result payload.

## Tradeoffs

- Docker backend security vs startup latency:
  - safer isolation with higher cold-start overhead than pure process execution.
- In-memory queue/store simplicity vs durability:
  - fast local operation, but requires external queue/store for cross-node durability.
- Process backend convenience vs safety:
  - useful for local development; not safe for untrusted multi-tenant production.
- Single-service deployment simplicity vs independent scaling:
  - API and workers run together now; can be split into dedicated services later.

## Run

```bash
cargo run
```

### Key Environment Variables

- `BIND_ADDR` (default `0.0.0.0:8080`)
- `WORKER_COUNT` (default `4`)
- `QUEUE_CAPACITY` (default `1024`)
- `API_KEYS` (format: `tenant:key,tenant2:key2`)
- `SANDBOX_BACKEND` (`docker` or `process`)
- `RATE_LIMIT_PER_MINUTE` (default `120`)
- `RATE_LIMIT_BURST` (default `20`)
- `NETWORK_ALLOWED_TENANTS` (comma-separated tenant IDs)
- `PERSIST_RESULTS_PATH` (optional JSONL path)
- `DEFAULT_CPU_CORES`, `DEFAULT_MEMORY_MB`, `DEFAULT_TIMEOUT_MS`, `DEFAULT_MAX_PROCESSES`
