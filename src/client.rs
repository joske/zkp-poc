use num_bigint::BigUint;
use zkp_poc::{calculate_response, decode, encode, modpow, random_number, G, H, P, Q};

use crate::auth::{
    auth_client::AuthClient, AuthenticationAnswerRequest, AuthenticationChallengeRequest,
    RegisterRequest,
};

pub mod auth {
    tonic::include_proto!("zkp_auth");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::args()
        .nth(2)
        .unwrap_or("http://127.0.0.1:50051".to_string());
    println!("Connecting to {}", url);
    let mut client = AuthClient::connect(url).await?;

    // hardcoded user
    let user = "zaphod".to_string();
    // very secret private key
    let x = BigUint::from(42u32);

    // BigUint::from can not be const, so stored as &[u8] and decoded here
    let g = decode(G);
    let h = decode(H);
    let p = decode(P);
    let q = decode(Q);
    let y1 = modpow(&g, &x, &p);
    let y2 = modpow(&h, &x, &p);

    let request = tonic::Request::new(RegisterRequest {
        user: user.clone(),
        y1: encode(&y1),
        y2: encode(&y2),
    });

    let _response = client.register(request).await?;

    // this should be a nonce, but we're running it only once here anyway
    let k = random_number();
    let r1 = modpow(&g, &k, &p);
    let r2 = modpow(&h, &k, &p);
    let request = tonic::Request::new(AuthenticationChallengeRequest {
        user,
        r1: encode(&r1),
        r2: encode(&r2),
    });

    let response = client.create_authentication_challenge(request).await?;
    let inner = response.into_inner();
    let auth_id = inner.auth_id;
    let challenge = decode(inner.c.as_slice());
    let s = calculate_response(&x, &k, &challenge, &q);

    let request = tonic::Request::new(AuthenticationAnswerRequest {
        auth_id,
        s: encode(&s),
    });

    let response = client.verify_authentication(request).await;
    match response {
        Ok(response) => println!("SUCCESS! session_id: {} ", response.into_inner().session_id),
        Err(e) => {
            println!("FAILURE: {e}");
            return Err("Authentication failed".into());
        }
    }

    Ok(())
}
