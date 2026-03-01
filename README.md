## Sandboxed Code Execution Engine

- Runs untrusted `python`, `javascript`, `rust` and `c` code behind a multi-tenant HTTP API
- Uses bounded queue + worker pool + sandbox backend (`docker` or `process`)
- Enforces per-run limits (CPU, memory, timeout, process count, file/output size)

### Architecture

- Request flow:
  `Client -> API (auth + validation + rate limit) -> Bounded Queue -> Worker Pool -> Sandbox -> Store`
- Storage:
  in-memory execution records, with optional JSONL persistence
- Isolation:
  API-key tenant auth + per-tenant rate limiting + optional network allowlist

### API

- Auth header: `x-api-key` (default key setup: `API_KEYS=default:dev-key`)
- Endpoints:
  - `GET /healthz` - health check
  - `GET /metrics` - Prometheus metrics
  - `POST /v1/executions` - submit execution
  - `GET /v1/executions/{id}` - execution status
  - `GET /v1/executions/{id}/result` - full record/result


### Configuration

- Runtime:
  - `BIND_ADDR` (`0.0.0.0:8080`)
  - `WORKER_COUNT` (`4`)
  - `QUEUE_CAPACITY` (`1024`)
  - `SANDBOX_BACKEND` (`docker`)
  - `LOG_LEVEL` (`info`)
- Limits defaults:
  - `DEFAULT_CPU_CORES` (`0.5`)
  - `DEFAULT_MEMORY_MB` (`256`)
  - `DEFAULT_TIMEOUT_MS` (`3000`)
  - `DEFAULT_MAX_PROCESSES` (`32`)
  - `DEFAULT_MAX_FILE_SIZE_BYTES` (`1048576`)
  - `DEFAULT_MAX_OUTPUT_BYTES` (`65536`)
- Multi-tenant and safety:
  - `API_KEYS` (`default:dev-key`; format: `tenant:key,tenant2:key2`)
  - `RATE_LIMIT_PER_MINUTE` (`120`)
  - `RATE_LIMIT_BURST` (`20`)
  - `NETWORK_ALLOWED_TENANTS` (empty by default)
  - `PERSIST_RESULTS_PATH` (unset by default)

## Structure

```text
.
|- Cargo.toml
|- Cargo.lock
|- README.md
|- src
|  |- main.rs
|  `- engine
|     |- mod.rs
|     |- api.rs
|     |- config.rs
|     |- error.rs
|     |- metrics.rs
|     |- models.rs
|     |- queue.rs
|     |- rate_limit.rs
|     |- store.rs
|     |- worker.rs
|     `- sandbox
|        |- mod.rs
|        |- docker.rs
|        |- process.rs
|        `- language.rs
`- target
```
