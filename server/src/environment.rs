use std::{fs, path::PathBuf};

use log::{debug, error, info};

use common::{group::TaskGroup};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct Environment {
    pub groups: Vec<TaskGroup>,
    pub log: Option<PathBuf>,
    pub dirty: bool
}

impl Serialize for Environment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        #[derive(Serialize)]
        struct SerializedEnvironment<'a> {
            pub groups: &'a Vec<TaskGroup>,
            pub log: &'a Option<PathBuf>,
        }

        SerializedEnvironment {
            groups: &self.groups,
            log: &self.log,
        }.serialize(serializer)
    }
}

impl Environment {
    pub fn update(&mut self) {
        debug!("[ENV] Update");
        for group in self.groups.iter_mut() {
            group.update();
        }

        if self.dirty {
            //TODO: Change
            if let Err(e) = fs::write("config.json",
                serde_json::to_string(self).unwrap()
            ) {
                error!("Unable to save the config: {}", e);
            } else {
                info!("Sucessfully saved the configuration");
                self.dirty = false;
            }
        }
    }

    fn get_task_group_log_path(path: &PathBuf, id: usize) -> PathBuf{
        path.join(id.to_string())
    }

    pub fn add_new_group(&mut self, mut task_group: TaskGroup) {
        let id = self.groups.len();

        if let Some(path) = &self.log {
            let group_path = Self::get_task_group_log_path(&path, id);
            task_group.set_log_path(group_path);
        }

        self.groups.push(task_group);
        self.dirty = true;
    }

    pub fn set_log_path(&mut self, path: PathBuf) {
        if !path.exists() {
            std::fs::create_dir(&path).unwrap();
        }

        for (id, group) in self.groups.iter_mut().enumerate() {
            let group_path = Self::get_task_group_log_path(&path, id);
            group.set_log_path(group_path);
        }
        self.log = Some(path);
    }
}