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
        .with_derive_builder(&["RawQueryRequest", "QueryRequest", "TimeQuery"], None)
        .with_field_attributes(
            &["TimeQuery.before", "TimeQuery.after"],
            &[r#"#[builder(setter(into, strip_option))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.timestamps"],
            &[r#"#[builder(setter(each(name="timestamp", into)))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.ids"],
            &[r#"#[builder(setter(each(name="id", into)))]"#],
        )
        .compile(
            &[
                "../protos/user-stats/messages.proto",
                "../protos/user-stats/rpc.proto",
            ],
            &["../protos"],
        )?;
    Ok(())
}
