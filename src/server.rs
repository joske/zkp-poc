use num_bigint::BigUint;
use std::{collections::HashMap, sync::Mutex};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
use zkp_auth::auth_server::{Auth, AuthServer};
use zkp_poc::{decode, encode, random_number, verify, G, H, P};

pub mod zkp_auth {
    tonic::include_proto!("zkp_auth");
}

#[derive(Debug, Default)]
struct UserInfo {
    y1: BigUint,
    y2: BigUint,
    r1: BigUint,
    r2: BigUint,
    c: BigUint,
}

#[derive(Debug, Default)]
pub struct MyAuth {
    /// Map user_id to UserInfo
    users: Mutex<HashMap<String, UserInfo>>,
    /// Map auth_id to user_id
    challenges: Mutex<HashMap<String, String>>,
}

#[tonic::async_trait]
impl Auth for MyAuth {
    async fn register(
        &self,
        request: Request<zkp_auth::RegisterRequest>,
    ) -> Result<Response<zkp_auth::RegisterResponse>, Status> {
        let user_info = UserInfo {
            y1: decode(request.get_ref().y1.as_slice()),
            y2: decode(request.get_ref().y2.as_slice()),
            ..Default::default()
        };
        self.users
            .lock()
            .map_err(|_| Status::internal("failed to get lock"))?
            .insert(request.get_ref().user.clone(), user_info);

        println!("User {} registered!", request.get_ref().user);
        let reply = zkp_auth::RegisterResponse {};

        Ok(Response::new(reply))
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<zkp_auth::AuthenticationChallengeRequest>,
    ) -> Result<Response<zkp_auth::AuthenticationChallengeResponse>, Status> {
        let user_id = request.get_ref().user.clone();
        let users = &mut self
            .users
            .lock()
            .map_err(|_| Status::internal("failed to get lock"))?;
        let Some(user_info) = users.get_mut(&user_id) else {
            return Err(Status::not_found("User not found"));
        };

        let auth_id: String = Uuid::new_v4().to_string();
        let c = random_number();

        user_info.c = c;
        user_info.r1 = decode(request.get_ref().r1.as_slice());
        user_info.r2 = decode(request.get_ref().r2.as_slice());

        self.challenges
            .lock()
            .map_err(|_| Status::internal("failed to get lock"))?
            .insert(auth_id.clone(), user_id.clone());
        let reply = zkp_auth::AuthenticationChallengeResponse {
            auth_id,
            c: encode(&user_info.c),
        };
        println!("User {} challenged!", user_id);

        Ok(Response::new(reply))
    }

    async fn verify_authentication(
        &self,
        request: Request<zkp_auth::AuthenticationAnswerRequest>,
    ) -> Result<Response<zkp_auth::AuthenticationAnswerResponse>, Status> {
        let challenges = &mut self
            .challenges
            .lock()
            .map_err(|_| Status::internal("failed to get lock"))?;

        let Some(user_id) = challenges.get_mut(&request.get_ref().auth_id) else {
            return Err(Status::not_found("Challenge not found"));
        };

        let users = &mut self
            .users
            .lock()
            .map_err(|_| Status::internal("failed to get lock"))?;
        let Some(user_info) = users.get_mut(user_id) else {
            return Err(Status::not_found("User not found"));
        };

        let g = decode(G);
        let h = decode(H);
        let p = decode(P);
        if !verify(
            &decode(request.into_inner().s.as_slice()),
            &user_info.r1,
            &user_info.r2,
            &user_info.c,
            &user_info.y1,
            &user_info.y2,
            &g,
            &h,
            &p,
        ) {
            return Err(Status::unauthenticated("Authentication failed"));
        }

        let session_id = Uuid::new_v4().to_string();
        // TODO: store session_id for user_id, but we're only doing the handshake POC here
        let reply = zkp_auth::AuthenticationAnswerResponse { session_id };

        println!("User {} authenticated!", user_id);
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let auth = MyAuth::default();

    Server::builder()
        .add_service(AuthServer::new(auth))
        .serve(addr)
        .await?;

    Ok(())
}
