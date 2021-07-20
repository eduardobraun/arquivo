use arquivo::{ArquivoClient, InsertRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::FmtSubscriber::builder()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let mut client = ArquivoClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(InsertRequest { namespace: "".to_string(), collection: "".to_string(), data: vec![] });

    tracing::info!(
        message = "Sending request.",
        // request = %request.get_ref().name
    );

    let _response = client.insert(request).await?;

    tracing::info!(
        message = "Got a response.",
        // response = %response.get_ref().message
    );
    Ok(())
}
