use serde::{Deserialize, Serialize};

use crate::group::SerializedTaskGroup;

#[derive(Deserialize, Serialize)]
pub enum Queries {
    Ok,
    NewTaskGroup(SerializedTaskGroup)
}
