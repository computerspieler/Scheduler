
use std::{
    fmt::{self, Formatter}, fs, path::PathBuf, sync::{Arc, RwLock}, thread::{self, JoinHandle}, time::Duration
};

use serde::{Deserialize, Serialize};
use log::{debug, warn, error};

use crate::command::*;

#[derive(Debug, Default)]
pub struct TaskStatistic {
    count: usize,
    error_count: usize,
    average_duration: Duration
}

impl fmt::Display for TaskStatistic {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "=== Statistics ===")?;
        writeln!(fmt, "Execution count: {}", self.count)?;
        writeln!(fmt, "Error rate: {}%", 100. * (self.error_count as f64) / (self.count as f64))?;
        write!(fmt, "Average execution time: {:?}", self.average_duration)?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub cmd: Command,
    pub max_concurrent_execution: Option<usize>,

    #[serde(skip_deserializing)]
    pub stdout_path: Option<PathBuf>,
    #[serde(skip_deserializing)]
    pub stderr_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Task {
    config: Arc<RwLock<TaskConfig>>,

    executions: Vec<TaskOutput>,
    running_threads: Vec<(usize, JoinHandle<TaskOutput>)>,
    stats: TaskStatistic,
}

impl Task {
    pub fn new(conf: TaskConfig) -> Self {
        let running_threads = {
            if let Some(max) = (&conf).max_concurrent_execution {
                Vec::with_capacity(max)
            } else {
                Vec::new()
            }
        };

        Self {
            config: Arc::new(RwLock::new(conf)),
            executions: Vec::new(),
            running_threads: running_threads,
            stats: TaskStatistic::default(),
        }
    }

    pub fn config(&self) -> TaskConfig {
        (*self.config).read().unwrap().clone()
    }

    pub fn set_log_path(&mut self, path: PathBuf) {
        if !path.exists() {
            std::fs::create_dir(&path).unwrap();
        }

        let stdout_path = path.join("out");
        if !stdout_path.exists() {
            std::fs::create_dir(&stdout_path).unwrap();
        }
        let stderr_path = path.join("err");
        if !stderr_path.exists() {
            std::fs::create_dir(&stderr_path).unwrap();
        }
        
        let mut conf = self.config.write().unwrap();
        conf.stdout_path = Some(stdout_path);
        conf.stderr_path = Some(stderr_path);
    }
    
    fn update_log(&self, idx: usize, output: TaskOutput) -> TaskOutput {
        let mut res = output?;

        if let Some(path) = &self.config.read()?.stdout_path {
            if let Log::Buffer(log) = &res.stdout {
                let path = path.join(idx.to_string());
                fs::write(&path, log)?;
                res.stdout = Log::File(path);
            }
        }

        if let Some(path) = &self.config.read()?.stderr_path {
            if let Log::Buffer(log) = &res.stderr {
                let path = path.join(idx.to_string());
                fs::write(&path, log)?;
                res.stderr = Log::File(path);
            }
        }

        TaskOutput::NoError(res)
    }

    fn update_stats(&mut self, res: &TaskOutput) {
        if let TaskOutput::NoError(outcome) = res {
            let n = self.stats.count - self.stats.error_count;
            let n: u32 = n.try_into().unwrap();
            let n: f64 = n.try_into().unwrap();
            
            self.stats.average_duration =
                self.stats.average_duration.mul_f64(n / (n + 1.)) +
                outcome.duration.div_f64(n + 1.);
        } else {
            self.stats.error_count += 1;
        }

        self.stats.count += 1;
    }

    fn join(&mut self, handler: JoinHandle<TaskOutput>) -> TaskOutput {
        TaskOutput::NoError(handler.join().unwrap()?)
    }

    fn set_task_output(&mut self, idx: usize, output: TaskOutput) {
        debug!("Execution n°{} is over", idx);

        self.update_stats(&output);
        self.executions[idx] = self.update_log(idx, output);
    }

    pub fn run(&mut self) {
        let conf = self.config.clone();
        let idx = self.executions.len();
        
        debug!("Starting execution n°{}", idx);
        self.executions.push(TaskOutput::Waiting);

        if let Some(max) = conf.read().unwrap().max_concurrent_execution {
            let nb_concurrent_threads = self.running_threads.len();
            if nb_concurrent_threads >= max {
                self.set_task_output(idx, TaskOutput::TooManyThreadsError);
                error!("Can't start execution n° {}: Too many concurrent threads", idx);
                return;
            } else if nb_concurrent_threads == 9 * max / 10 {
                warn!("More than 90% of possible threads are running concurrently");
            }
        }

        self.running_threads.push((idx,
            thread::spawn(
                move || -> TaskOutput {
                    conf.read()?.cmd.run()
                }
            )
        ));
    }

    pub fn update(&mut self) -> bool {
        let mut has_thread_finished = false;
        let mut n = self.running_threads.len();
        let mut i = 0;

        while i < n {
            if self.running_threads[i].1.is_finished() {
                let (idx, handler) = self.running_threads.swap_remove(i);
                let res = self.join(handler);
                self.set_task_output(idx, res);
                n -= 1;
                has_thread_finished = true;
            } else {
                i += 1;
            }
        }

        has_thread_finished
    }

    pub fn nb_running_tasks(&self) -> usize {
        self.running_threads.len()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, TaskOutput> {
        self.executions.iter()
    }

    pub fn stats(&self) -> &TaskStatistic {
        &self.stats
    }
}