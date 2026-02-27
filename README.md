# rust-pubsub

Proof of concept: asynchronous service communication via Google Cloud Pub/Sub
from Rust.

Two [Axum](https://github.com/tokio-rs/axum) / Tokio services exchange messages
through Pub/Sub using the
[gcloud-pubsub](https://crates.io/crates/gcloud-pubsub) crate:

1. **api-service** — accepts loan applications over HTTP, publishes them to
   Pub/Sub, and lets users poll for results.
2. **risk-processor** — subscribes to applications, makes a random
   approve/reject decision, and publishes the result back.

## Architecture

```
                        ┌──────────────────┐
  POST /applications ──>│   api-service    │──> loan-applications topic
                        │  (HTTP + state)  │
  GET /applications/:id │                  │<── loan-decisions topic
          <─────────────└──────────────────┘

                        ┌──────────────────┐
  loan-applications ──> │ risk-processor   │──> loan-decisions topic
          topic         │  (subscriber)    │
                        └──────────────────┘
```

## Quick Start

```sh
-> % ./scripts/start.sh

Building and starting containers...
[+] Building 1.3s (31/31) FINISHED
(...)
Tailing logs (Ctrl+C to stop watching, containers keep running)...
Waiting for API service to become ready...

API service is ready at http://localhost:3000

-> % ./scripts/submit-application.sh u-7712 15000 PLN
{
  "application_id": "loan-24198"
}

-> % ./scripts/check-status.sh loan-24198
{
  "application_id": "loan-24198",
  "status": "APPROVED",
  "interest_rate": 6.73,
  "max_term_months": 22
}

% ./scripts/logs.sh

rust-pubsub-risk-processor-1  | Connected to Pub/Sub.
rust-pubsub-risk-processor-1  | Listening for loan applications on 'loan-applications-sub'...
rust-pubsub-risk-processor-1  | Processing application loan-24198 for user u-7712 (15000 PLN)
rust-pubsub-risk-processor-1  | Decision for loan-24198: Approved
rust-pubsub-api-service-1     | Connected to Pub/Sub.
rust-pubsub-api-service-1     | Listening on 0.0.0.0:3000
rust-pubsub-api-service-1     | Background listener subscribed to 'loan-decisions-sub'
rust-pubsub-api-service-1     | Submitting application loan-24198 for user u-7712 (15000 PLN)
rust-pubsub-api-service-1     | Received decision for loan-24198: Approved
```

## Running

Everything runs in Docker via the official
[Pub/Sub emulator](https://cloud.google.com/pubsub/docs/emulator):

```sh
./scripts/start.sh
```

Or directly:

```sh
docker compose up --build
```

This starts the emulator, creates topics and subscriptions, and launches both
services. The API listens on port 3000.

## API

**Submit an application**

```sh
./scripts/submit-application.sh u-7712 15000 PLN
```

```
POST /applications  {"user_id": "u-7712", "amount": 15000, "currency": "PLN"}
```

**Check application status**

```sh
./scripts/check-status.sh loan-12345
```

```
GET /applications/loan-12345
```

The response starts as `"status": "PENDING"` and changes to `"APPROVED"` or
`"REJECTED"` once the risk processor publishes its decision (typically 1-2
seconds).

## Limitations

Error handling is not production-ready. Both services use `?` to propagate
errors, which means the process exits on the first failure (e.g. a failed
publish or ack). A real system would need structured error types, retries for
transient failures, and dead-letter handling for poison messages.

## Stopping

```sh
./scripts/stop.sh
```
