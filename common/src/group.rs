use std::path::PathBuf;

use log::info;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{task::{Task, TaskConfig}, utils::{get_period_from_string, get_start_timestamp_from_string, YmdHmsDuration}};

#[derive(Debug)]
pub struct TaskGroup {
    name: String,
    starts_at: Option<DateTime<Utc>>,
    period: Option<YmdHmsDuration>,
    processes: Vec<Task>,

    next_execution: Option<DateTime<Utc>>
}

#[derive(Deserialize, Serialize)]
pub struct SerializedTaskGroup {
    name: String,
    starts_at: Option<String>,
    period: Option<String>,
    processes: Vec<TaskConfig>
}

impl<'de> Deserialize<'de> for TaskGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        Ok(TaskGroup::from(
            SerializedTaskGroup::deserialize(deserializer)?
        ))
    }
}

impl Serialize for TaskGroup {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        SerializedTaskGroup {
            name: self.name.clone(),
            starts_at: self.starts_at
                .map(|dt| { dt.to_string() }),
            period: self.period
                .as_ref()
                .map(|per| { per.to_string() }),
            processes: self.processes.iter()
                .map(|task| task.config())
                .collect()
        }.serialize(serializer)
    }
}

impl From<SerializedTaskGroup> for TaskGroup {
    fn from(conf: SerializedTaskGroup) -> Self {
        TaskGroup::new(
            conf.name,
            conf.starts_at.map(|x|
                get_start_timestamp_from_string(x.as_str())
                    .expect(format!("Invalid date: {}", x).as_str())
            ),
            conf.period.map(|x|
                get_period_from_string(x.as_str())
                    .expect(format!("Invalid period: {}", x).as_str())
            ),
            conf.processes.iter()
                .map(|conf| {
                    Task::new(conf.clone())
                })
                .collect()
        )
    }
}

impl TaskGroup {
    pub fn new(
        name: String,
        starts_at: Option<DateTime<Utc>>,
        period: Option<YmdHmsDuration>,
        processes: Vec<Task>
    ) -> Self {
        let mut out = Self {
            name: name,
            starts_at: starts_at,
            period: period,
            processes: processes,

            next_execution: None
        };

        if let Some(start) = starts_at {
            out.update_next_execution(start);
        }

        out
    }

    pub fn set_log_path(&mut self, path: PathBuf) {
        if !path.exists() {
            std::fs::create_dir(&path).unwrap();
        }

        for (id, task) in self.processes.iter_mut().enumerate() {
            let task_path = path.join(id.to_string());
            task.set_log_path(task_path);
        }
    }

    fn update_next_execution(&mut self, last_execution: DateTime<Utc>) {
        let now = Utc::now();
        match &self.period {
        None => self.next_execution =
            if last_execution > now {
                Some(last_execution)
            } else {
                None
            },
        Some(period) => {
            let mut next_execution = last_execution;
            while next_execution < now {
                next_execution = period.add(next_execution);
            }
            self.next_execution = Some(next_execution);
        }
        }
    }

    pub fn add_process(&mut self, task: Task) {
        self.processes.push(task);
    }

    pub fn update(&mut self) -> bool {
        let now = Utc::now();

        let mut has_anything_changed = false;

        info!("\"{}\": Updating", self.name);
        for task in self.processes.iter_mut() {
            has_anything_changed |= task.update();
        }

        if self.next_execution.is_none() {
            info!("\"{}\": No update planned", self.name);
            return has_anything_changed;
        }

        let next_execution = self.next_execution.unwrap();
        if next_execution > now {
            info!("\"{}\": Too early (it's {})", self.name, now);
            return has_anything_changed;
        }

        info!("\"{}\": Launching new tasks", self.name);
        self.update_next_execution(now);
        for task in self.processes.iter_mut() {
            task.run();
        }

        return true;
    }
}
