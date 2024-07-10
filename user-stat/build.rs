use std::fs;

use anyhow::Result;
use proto_builder_trait::tonic::BuilderAttributes;
fn main() -> Result<()> {
    let path = "src/pb";
    fs::create_dir_all(path)?;

    let config = tonic_build::configure();
    config
        .out_dir(path)
        .with_sqlx_from_row(&["User"], None)
        .compile(
            &[
                "../protos/user-stats/messages.proto",
                "../protos/user-stats/rpc.proto",
            ],
            &["../protos"],
        )?;
    Ok(())
}
