use tonic::{transport::Server, Request, Response, Status};
use zkp_auth::auth_server::{Auth, AuthServer};

pub mod zkp_auth {
    tonic::include_proto!("zkp_auth"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyAuth {}

#[tonic::async_trait]
impl Auth for MyAuth {
    async fn register(
        &self,
        request: Request<zkp_auth::RegisterRequest>,
    ) -> Result<Response<zkp_auth::RegisterResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = zkp_auth::RegisterResponse {};

        Ok(Response::new(reply))
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<zkp_auth::AuthenticationChallengeRequest>,
    ) -> Result<Response<zkp_auth::AuthenticationChallengeResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = zkp_auth::AuthenticationChallengeResponse {
            auth_id: "auth_id".to_string(),
            c: 1,
        };

        Ok(Response::new(reply))
    }

    async fn verify_authentication(
        &self,
        request: Request<zkp_auth::AuthenticationAnswerRequest>,
    ) -> Result<Response<zkp_auth::AuthenticationAnswerResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = zkp_auth::AuthenticationAnswerResponse {
            session_id: "session_id".to_string(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let auth = MyAuth::default();

    Server::builder()
        .add_service(AuthServer::new(auth))
        .serve(addr)
        .await?;

    Ok(())
}

