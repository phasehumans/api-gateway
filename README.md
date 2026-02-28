# Rust API Gateway

A modular, middleware-driven API gateway inspired by Kong-style internal abstractions.

## Features

- API key authentication (`x-api-key`) with timing-safe comparison
- Rate limiting with two algorithms:
  - Token bucket
  - Sliding window
- Rate limiting backends:
  - In-memory
  - Redis (atomic Lua scripts)
- Request logging with request IDs and latency
- Request validation (method allowlist, header limits, body size checks)
- Circuit breaking (closed/open/half-open)
- Intelligent upstream routing (weights + runtime load/failure/latency signals)
- Failover across ranked upstreams when an upstream call fails
- Security headers on responses

## Architecture

- `src/gateway.rs`: request lifecycle orchestration
- `src/middleware/*`: middleware contracts and implementations
- `src/ratelimit/*`: backend abstraction + in-memory and Redis implementations
- `src/upstream.rs`: forwarding client and upstream runtime metrics
- `src/router.rs`: intelligent routing strategy
- `src/circuit_breaker.rs`: resilience state machine
- `src/config.rs`: environment-driven config

## Configuration

Key environment variables:

- `BIND_ADDR` (default: `0.0.0.0:8080`)
- `API_KEYS` (default: `dev-key`)
- `AUTH_EXEMPT_PREFIXES` (default: `/health`)
- `UPSTREAMS` (default: `svc-a=http://127.0.0.1:9001,svc-b=http://127.0.0.1:9002`)
  - Format per upstream: `name=url@weight@timeout_ms`
- `ROUTES` (default: `/=svc-a|svc-b,/health=svc-a`)

Rate limiting:

- `RATE_LIMIT_ENABLED` (default: `true`)
- `RATE_LIMIT_BACKEND` (`memory` or `redis`)
- `RATE_LIMIT_ALGORITHM` (`token_bucket` or `sliding_window`)
- `RATE_LIMIT_KEY_HEADER` (default: `x-api-key`)
- `RATE_LIMIT_FAIL_OPEN` (default: `false`)

Token bucket:

- `RATE_LIMIT_CAPACITY` (default: `200`)
- `RATE_LIMIT_REFILL_TPS` (default: `100`)

Sliding window:

- `RATE_LIMIT_WINDOW_SECONDS` (default: `60`)
- `RATE_LIMIT_MAX_REQUESTS` (default: `600`)

Redis backend:

- `REDIS_URL` (default: `redis://127.0.0.1:6379`)
- `REDIS_KEY_PREFIX` (default: `gateway:ratelimit`)

Circuit breaker:

- `CB_FAILURE_THRESHOLD` (default: `5`)
- `CB_OPEN_SECONDS` (default: `20`)
- `CB_HALF_OPEN_MAX` (default: `1`)

Validation:

- `MAX_BODY_BYTES` (default: `1048576`)
- `ALLOWED_METHODS` (default: `GET,POST,PUT,PATCH,DELETE,OPTIONS`)
- `REQUIRE_HOST_HEADER` (default: `true`)
- `MAX_HEADERS` (default: `128`)

Routing tuning:

- `ROUTING_PREFER_LOW_LATENCY` (default: `true`)
- `ROUTING_IN_FLIGHT_PENALTY` (default: `12`)
- `ROUTING_FAILURE_PENALTY` (default: `250`)

## Run

```bash
cargo run
```

Then send requests with `x-api-key` set to a configured key.
