use std::path::PathBuf;

use log::info;
use serde::{de::Deserializer, Deserialize};

use crate::group::TaskGroup;

#[derive(Debug)]
pub struct Environment {
    groups: Vec<TaskGroup>
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        pub struct EnvironmentJson {
            log: Option<PathBuf>,
            groups: Vec<TaskGroup>
        }

        let val = EnvironmentJson::deserialize(deserializer)?;
        let mut output = Environment {
            groups: val.groups
        };

        if let Some(path) = val.log {
            output.set_log_path(path);
        }

        Ok(output)
    }
}

impl Environment {
    pub fn update(&mut self) {
        info!("[ENV] Update");
        for group in self.groups.iter_mut() {
            group.update();
        }
    }

    fn set_log_path(&mut self, path: PathBuf) {
        if !path.exists() {
            std::fs::create_dir(&path).unwrap();
        }

        for (id, group) in self.groups.iter_mut().enumerate() {
            let group_path = path
                .join(id.to_string());

            group.set_log_path(group_path);
        }
    }
}