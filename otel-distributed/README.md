# otel-distributed

A sample app showcasing distributed tracing with a Rust server using Axum and OpenTelemetry, and a NodeJS client.
Both client and server use Sentry for tracing and errors.

To run the example, set the `SENTRY_DSN` environment variable to the DSN for the Sentry project where you want to capture Rust telemetry, then run `make server` to compile and run the server.
Similarly, set `SENTRY_DSN` to the DSN for the Sentry project you want to capture NodeJS telemetry (could be the same or a different one than the first), then run `make client` to run the client.

Both the server and client will capture spans and errors that you will be able to view respectively in the Traces and Issues parts of the UI.
