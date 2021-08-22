use arquivo::{ArquivoServer, ArquivoSvc};
use dotenv::dotenv;
use opentelemetry::sdk::trace::Tracer;
use tokio::signal::unix::{signal, SignalKind};
use tonic::transport;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Registry};

fn has_jaeger_env() -> bool {
    const ENV_AGENT_HOST: &str = "OTEL_EXPORTER_JAEGER_AGENT_HOST";
    const ENV_ENDPOINT: &str = "OTEL_EXPORTER_JAEGER_ENDPOINT";

    if let Some(s) = dotenv::var(ENV_AGENT_HOST)
        .or_else(|_| dotenv::var(ENV_ENDPOINT))
        .ok()
    {
        return !s.is_empty();
    }
    false
}

fn create_jaeger_layer<S>() -> Option<OpenTelemetryLayer<S, Tracer>>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    if !has_jaeger_env() {
        return None;
    }

    match opentelemetry_jaeger::new_pipeline().install_simple() {
        Ok(tracer) => Some(tracing_opentelemetry::layer().with_tracer(tracer)),
        Err(_) => None,
    }
}

fn setup_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let fmt_layer = fmt::Layer::new();
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    let subscriber = Registry::default().with(filter_layer).with(fmt_layer);

    let telemetry_layer = create_jaeger_layer();
    let subscriber = subscriber.with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    setup_tracing()?;

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

    let addr = dotenv::var("RPC_ADDR")
        .ok()
        .unwrap_or("[::1]:50051".to_string())
        .parse()?;
    let arquivo_service = arquivo::ArquivoServer::new(arquivo::ArquivoSvc::new());

    health_reporter
        .set_serving::<ArquivoServer<ArquivoSvc>>()
        .await;

    let sig_int = signal(SignalKind::interrupt()).unwrap();
    let sig_term = signal(SignalKind::terminate()).unwrap();
    // must pin to use multiple times (tokio::select)
    tokio::pin!(sig_int);
    tokio::pin!(sig_term);

    let err = transport::Server::builder()
        .trace_fn(|_| tracing::span!(tracing::Level::TRACE, "arquivo_server"))
        .add_service(health_service)
        .add_service(arquivo_service)
        .serve_with_shutdown(addr, async move {
            tokio::select! {
                _ = sig_int.recv() => {},
                _ = sig_term.recv() => {},
            };
            tracing::info!("Shutdown signal received");
        })
        .await;
    health_reporter
        .set_not_serving::<ArquivoServer<ArquivoSvc>>()
        .await;

    match err {
        Ok(_) => (),
        Err(e) => tracing::error!("server error: {}", e),
    };

    tracing::info!("Bye, bye!");
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
