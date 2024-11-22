pub use tracing;
use tracing::{level_filters::LevelFilter, subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation notes
///
/// We are using `impl Subscriber` as return type to avoid having
/// to spell out the actual type, which is indeed quite complex.
/// We need to explicitely call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: LevelFilter,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter.to_string()));

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// Note: this should only be called once.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to initialize logger.");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

/// A trait for errors that can be traced.
///
/// This trait is implemented for all types that satisfy the bounds
/// `std::error::Error + Into<anyhow::Error>`.
pub trait TracedError: std::error::Error + Into<anyhow::Error> {
    fn traced<E>(self) -> impl FnOnce(E) -> anyhow::Error
    where
        E: std::fmt::Debug + std::fmt::Display,
        Self: Sized,
    {
        move |error| {
            tracing::error!("{:?}", error);
            self.into()
        }
    }
}

impl<T> TracedError for T where T: std::error::Error + Into<anyhow::Error> {}
