use std::fs;

use anyhow::Result;
use prost_build::Config;
fn main() -> Result<()> {
    let path = "src/pb";
    fs::create_dir_all(path)?;
    let mut config = Config::new();
    config
        .out_dir(path)
        .compile_protos(&["../protos/crm.proto"], &["../protos"])?;
    Ok(())
}
