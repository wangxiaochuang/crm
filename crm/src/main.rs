use anyhow::Result;
use crm::pb;
use prost::Message as _;

fn main() -> Result<()> {
    let user = pb::User::new(1, "jack@gmail.com".to_string(), "jack".to_string());
    let encoded = user.encode_to_vec();
    let newuser = pb::User::decode(encoded.as_ref())?;
    println!("{:?}, encoded: {:?}, {:?}", user, encoded, newuser);
    Ok(())
}
