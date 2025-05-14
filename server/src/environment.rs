use std::path::PathBuf;

use log::debug;

use common::{group::TaskGroup};

#[derive(Debug)]
pub struct Environment {
    pub groups: Vec<TaskGroup>,
    pub log: Option<PathBuf>,
}

impl Environment {
    pub fn update(&mut self) {
        debug!("[ENV] Update");
        for group in self.groups.iter_mut() {
            group.update();
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

        self.groups.push(task_group)
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