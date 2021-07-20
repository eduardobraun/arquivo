use arquivo::{ArquivoServer, ArquivoSvc};
use dotenv::dotenv;
use tokio::signal::unix::{signal, SignalKind};
use tonic::transport;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let fmt_layer = fmt::layer();
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    let subscriber = Registry::default().with(filter_layer).with(fmt_layer);

    // TODO: check for telemetry env vars
    let telemetry_layer = if true {
        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_agent_endpoint("localhost:6831")
            .with_service_name("arquivo_svc")
            .with_max_packet_size(1_000_000)
            .install_simple()?;

        Some(tracing_opentelemetry::layer().with_tracer(tracer))
    } else {
        None
    };
    let subscriber = subscriber.with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    let sig_int = signal(SignalKind::interrupt()).unwrap();
    let sig_term = signal(SignalKind::terminate()).unwrap();
    // must pin to use multiple times (tokio::select)
    tokio::pin!(sig_int);
    tokio::pin!(sig_term);

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

    let addr = dotenv::var("RPC_ADDR")
        .ok()
        .unwrap_or("[::1]:50051".to_string())
        .parse()?;
    let arquivo_service = arquivo::ArquivoServer::new(arquivo::ArquivoSvc::new());

    health_reporter
        .set_serving::<ArquivoServer<ArquivoSvc>>()
        .await;
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
