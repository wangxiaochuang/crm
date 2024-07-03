use std::time::SystemTime;

use prost_types::Timestamp;

use crate::pb::User;

impl User {
    pub fn new(id: u64, email: String, name: String) -> Self {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        Self {
            id,
            email,
            name,
            created_at: Some(Timestamp {
                seconds: ts.as_secs() as i64,
                nanos: ts.subsec_nanos() as i32,
            }),
        }
    }
}
