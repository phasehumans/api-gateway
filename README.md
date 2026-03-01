## Sandboxed Code Execution Engine

Run untrusted `python`/`javascript`/`rust`/`c` code behind a multi-tenant API with queueing, worker isolation, and strict runtime limits.

```
Client -> API (Auth + Validation + Rate Limit) -> Bounded Queue -> Worker Pool
                                                                    |
Docker Sandbox (or Process) -> Execution Store (In-memory + optional JSONL)
```

### Why this exists

- Isolate untrusted execution with per-run CPU, memory, timeout, process, and output limits
- Keep API responsive with bounded queue and async workers
- Support tenant isolation with API-key auth and per-tenant rate limiting
- Expose operational metrics via Prometheus format

### Endpoints

- `GET /healthz` - Health check
- `GET /metrics` - Prometheus metrics
- `POST /v1/executions` - Submit execution job
- `GET /v1/executions/{id}` - Job status
- `GET /v1/executions/{id}/result` - Full result

Auth via `x-api-key` header. Default: `API_KEYS=default:dev-key`

### Configuration

- `BIND_ADDR` (default `0.0.0.0:8080`) - HTTP bind address
- `WORKER_COUNT` (default `4`) - Number of worker tasks
- `QUEUE_CAPACITY` (default `1024`) - Bounded queue size
- `SANDBOX_BACKEND` (default `docker`) - `docker` or `process`
- `API_KEYS` (default `default:dev-key`) - Format: `tenant:key,tenant2:key2`
- `RATE_LIMIT_PER_MINUTE` (default `120`) - Per-tenant request rate
- `RATE_LIMIT_BURST` (default `20`) - Token bucket burst size
- `NETWORK_ALLOWED_TENANTS` (empty) - Tenants allowed `allow_network=true`
- `PERSIST_RESULTS_PATH` (unset) - Append JSONL results to disk
- `LOG_LEVEL` (default `info`) - Tracing filter
