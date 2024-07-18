use std::fs;

use anyhow::Result;
use proto_builder_trait::tonic::BuilderAttributes;
fn main() -> Result<()> {
    let path = "src/pb";
    fs::create_dir_all(path)?;

    let config = tonic_build::configure();
    config
        .out_dir(path)
        .with_type_attributes(&["MaterializeRequest"], &[r#"#[derive(Eq, Hash)]"#])
        .compile(
            &[
                "../protos/metadata/messages.proto",
                "../protos/metadata/rpc.proto",
            ],
            &["../protos"],
        )?;
    Ok(())
}
