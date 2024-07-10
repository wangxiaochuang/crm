use std::fs;

use anyhow::Result;
fn main() -> Result<()> {
    let path = "src/pb";
    fs::create_dir_all(path)?;
    let config = tonic_build::configure();
    config
        .out_dir(path)
        .compile(&["../protos/crm/crm.proto"], &["../protos"])?;
    Ok(())
}
