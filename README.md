# Cron Runner (Rust)

Ultra-light HTTP scheduler in Rust with **per-second cron** precision.  
Adds `X-Cron-Secret` to every request, supports custom headers and optional body per job.

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/deploy/cron-rust?referralCode=1q5cCO&utm_medium=integration&utm_source=template&utm_campaign=generic)

## Features
- 6-field cron expressions (seconds included).
- HTTP methods: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`.
- Per-job **headers** and **body** (raw string).
- Minimal sync HTTP client (`ureq`).
- Structured logs: success (`OK`) with status, failures (`FAIL`) with error cause.

## Deploy and Host
Use your preferred workflow (Cargo, Docker, or Railway). The service runs as a **persistent process** and schedules requests internally—no external cron required. When hosting on Railway, set the variables below and deploy in a region close to your target APIs to minimize latency.

## Why Deploy
- Replace heavyweight schedulers with a tiny Rust binary.
- Run cron **with second-level precision**.
- Centralize multiple HTTP jobs (headers, bodies, secrets) in one place.
- Keep simple, transparent logs for success/failure.

## Common Use Cases
- Pinging internal or public endpoints on a schedule.
- Triggering background jobs (Laravel/Symfony/NestJS/Rails) via authenticated webhooks.
- Refreshing caches or tokens; pre-warming data.
- Kicking off ETL or status checks at fine-grained intervals.

## Environment Variables
- `SECRET` **(required)** — shared secret sent as `X-Cron-Secret` on every request.
- `CRON_JOBS` **(required)** — list of jobs separated by **`;` or new lines**.  
  **Format:** `METHOD|URL|CRON_EXPR|HEADERS|BODY`  
  - `METHOD`: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`  
  - `URL`: full HTTP(S) endpoint (query params allowed)  
  - `CRON_EXPR`: **6-field cron** (with seconds)  
  - `HEADERS` *(optional)*: comma-separated `Name:Value` pairs  
  - `BODY` *(optional)*: raw string payload

**Examples**
```
GET|https://httpbingo.org/status/204|0 * * * * *||
POST|https://httpbin.org/post|*/30 * * * * *|Content-Type:application/json|{"ping":true}
PATCH|https://postman-echo.com/patch|15 * * * * *|Authorization:Bearer XYZ,Content-Type:application/json|{"name":"demo"}
DELETE|https://postman-echo.com/delete|45 * * * * *||
```

## Quick Start

### Cargo
```bash
export SECRET=mysecret
export CRON_JOBS='GET|https://httpbingo.org/status/204|0 * * * * *||;POST|https://httpbin.org/post|*/30 * * * * *|Content-Type:application/json|{"ping":true}'
cargo run --release
```

### Docker
```bash
docker build -t cron-runner .
docker run --rm   -e SECRET=mysecret   -e CRON_JOBS='GET|https://httpbingo.org/status/204|0 * * * * *||;POST|https://httpbin.org/post|*/30 * * * * *|Content-Type:application/json|{"ping":true}'   cron-runner
```

### Railway (tips)
- Use **Reference Variables** to call another service over the private network:  
  `http://${{service.RAILWAY_PRIVATE_DOMAIN}}:${{service.PORT}}/...`
- Private networking avoids public egress and can be safer for service-to-service calls.

## Logs
- OK: `YYYY-MM-DDTHH:MM:SSZ | OK | METHOD URL | STATUS`
- FAIL: `YYYY-MM-DDTHH:MM:SSZ | FAIL | METHOD URL | HTTP XXX (client/server error)` or transport error.

## Tech
- Rust + `ureq` (sync, minimal) for HTTP.
- `cron` crate for per-second schedules.

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/deploy/cron-rust?referralCode=1q5cCO&utm_medium=integration&utm_source=template&utm_campaign=generic)


