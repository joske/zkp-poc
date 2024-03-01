use crate::auth::{auth_client::AuthClient, RegisterRequest};

pub mod auth {
    tonic::include_proto!("zkp_auth");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AuthClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(RegisterRequest {
        user: "user".to_string(),
        y1: 1i64,
        y2: 2i64,
    });

    let response = client.register(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

