fn main() {
    println!("Hello, world!");
}

#[test]
fn test_init() {
    let _guard = sentry::init((
        "https://fff@o0.ingest.us.sentry.io/0",
        sentry::ClientOptions {
            auto_session_tracking: true,
            session_mode: sentry::SessionMode::Application,
            ..Default::default()
        },
    ));
}
