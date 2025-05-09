use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use opentelemetry::trace::{get_active_span, TraceContextExt};
use opentelemetry::{
    global,
    propagation::Extractor,
    trace::{Span, SpanKind, Status, Tracer},
    Context,
};
use opentelemetry_sdk::trace::SdkTracerProvider;
use sentry::integrations::opentelemetry::{SentryPropagator, SentrySpanProcessor};
use serde::{Deserialize, Serialize};

// Header extractor for OpenTelemetry context propagation
struct HeaderExtractor<'a> {
    headers: &'a HeaderMap,
}

impl<'a> HeaderExtractor<'a> {
    fn new(headers: &'a HeaderMap) -> Self {
        HeaderExtractor { headers }
    }
}

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|k| k.as_str()).collect()
    }
}

fn main() {
    // Initialize the Sentry SDK
    let _guard = sentry::init(sentry::ClientOptions {
        traces_sample_rate: 1.0,
        in_app_include: vec!["rust_server"],
        in_app_exclude: vec![""],
        debug: true,
        ..sentry::ClientOptions::default()
    });

    // Register the Sentry propagator to enable distributed tracing
    global::set_text_map_propagator(SentryPropagator::new());

    // Create a tracer provider with the Sentry span processor
    let tracer_provider = SdkTracerProvider::builder()
        .with_span_processor(SentrySpanProcessor::new())
        .build();

    // Set the global tracer provider
    global::set_tracer_provider(tracer_provider);

    println!("OpenTelemetry initialized with Sentry");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Build our application with routes
            let app = Router::new().route("/hello", get(hello));

            println!("Starting server on 127.0.0.1:3001");

            // Run the server
            let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
                .await
                .unwrap();
            axum::serve(listener, app).await.unwrap();
        });
}

// Validate the input parameters
fn validate_message(params: &HelloParams) -> Result<(), AppError> {
    // Get the current active context
    let current_context = Context::current();
    let tracer = global::tracer("hello_handler");

    let validation_span = tracer
        .span_builder("validate_params")
        .with_kind(SpanKind::Internal)
        .start_with_context(&tracer, &current_context);

    // Set validation span as current
    let validation_cx = Context::current_with_span(validation_span);
    let _validation_guard = validation_cx.attach();

    // Simulate some work
    tokio::task::block_in_place(|| {
        std::thread::sleep(std::time::Duration::from_millis(50));
    });

    // Check if the message is valid
    if params.message.is_empty() {
        // Mark span as error
        get_active_span(|span| {
            span.set_status(Status::Error {
                description: "Message cannot be empty".into(),
            });
        });

        println!("Error: Message cannot be empty");

        let err = AppError::InvalidInput("Message cannot be empty".to_string());

        sentry::capture_error(&err);

        return Err(err);
    }

    Ok(())
}

async fn hello(
    headers: HeaderMap,
    params: Query<HelloParams>,
) -> Result<impl IntoResponse, AppError> {
    // Extract the OpenTelemetry context from request headers
    let parent_context =
        global::get_text_map_propagator(|prop| prop.extract(&HeaderExtractor::new(&headers)));

    // Create a tracer and start a span
    let tracer = global::tracer("handle_hello");
    let span = tracer
        .span_builder("handle_hello")
        .with_kind(SpanKind::Server)
        .with_attributes(vec![
            opentelemetry::KeyValue::new("params", format!("{:?}", params)),
            opentelemetry::KeyValue::new("headers", format!("{:?}", headers)),
        ])
        .start_with_context(&tracer, &parent_context);

    // Set the span as current
    let cx = Context::current_with_span(span);
    let _guard = cx.attach();

    println!("Hello endpoint hit with params: {:?}", params);
    println!("Request headers: {:?}", headers);

    // Simulate some work
    tokio::task::block_in_place(|| {
        std::thread::sleep(std::time::Duration::from_millis(200));
    });

    // Validate the input
    if let Err(e) = validate_message(&params) {
        return Err(e);
    }

    // Create a child span for work simulation
    let mut work_span = tracer
        .span_builder("simulate_work")
        .with_kind(SpanKind::Internal)
        .start(&tracer);

    // Mark work span as successful
    work_span.set_status(Status::Ok);

    // Set work span as current
    let work_cx = Context::current_with_span(work_span);
    let _work_guard = work_cx.attach();

    // Simulate some work
    tokio::task::block_in_place(|| {
        std::thread::sleep(std::time::Duration::from_millis(200));
    });

    Ok(Json(HelloResponse {
        message: format!("Hello from Rust! You sent: {}", params.message),
    }))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    #[serde(default)]
    message: String,
}

#[derive(Debug, Serialize)]
struct HelloResponse {
    message: String,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string(),
        }));

        (status, body).into_response()
    }
}
