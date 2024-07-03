use anyhow::Result;
use crm::pb::{user_service_client::UserServiceClient, CreateUserRequest};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50050").await?;

    let request = tonic::Request::new(CreateUserRequest {
        name: "jack".into(),
        email: "jack@gmail.com".into(),
    });

    let response = client.create_user(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
