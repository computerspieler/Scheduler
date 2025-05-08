use std::{
    fs, sync::{Arc, RwLock}, thread::{self, JoinHandle}, time::Duration
};

use crate::command::*;

#[derive(Debug, Default)]
pub struct TaskStatistic {
    count: usize,
    error_count: usize,
    average_duration: Duration
}

#[derive(Debug)]
pub struct Task {
    cmd: Arc<RwLock<Command>>,

    executions: Vec<TaskOutput>,
    running_threads: Vec<(usize, JoinHandle<TaskOutput>)>,
    stats: TaskStatistic,

    log_path: Option<String>
}

impl Task {
    pub fn new(cmd: Command, log_path: String) -> Self {
        Self {
            cmd: Arc::new(RwLock::new(cmd)),

            executions: Vec::new(),
            running_threads: Vec::new(),
            stats: TaskStatistic::default(),

            log_path: Some(log_path)
        }
    }
    
    fn update_log(&self, mut res: CommandOutcome, idx: usize) -> TaskOutput {
        if let Some(path) = &self.log_path {
            if let Log::Buffer(log) = &res.stdout {
                let path = format!("{}/{}.out", path, idx);
                fs::write(&path, log)?;
                res.stdout = Log::File(path);
            }
            if let Log::Buffer(log) = &res.stderr {
                let path = format!("{}/{}.err", path, idx);
                fs::write(&path, log)?;
                res.stderr = Log::File(path);
            }
        }
        TaskOutput::NoError(res)
    }

    fn update_stats(&mut self, res: &TaskOutput) {
        let n = self.stats.count;
        self.stats.count += 1;

        if let TaskOutput::NoError(outcome) = res {
            let n: u32 = n.try_into().unwrap();
            let n: f64 = n.try_into().unwrap();
            
            self.stats.average_duration =
                self.stats.average_duration.mul_f64(n / (n + 1.)) +
                outcome.duration.div_f64(n + 1.);
        }

        if res.is_error() {
            self.stats.error_count += 1;
        }
    }

    fn update_log_and_stats(&mut self, handler: JoinHandle<TaskOutput>, idx: usize) -> TaskOutput {
        let res = self.update_log(handler.join()??, idx);
        self.update_stats(&res);
        res
    }

    pub fn run(&mut self) {
        let cmd = self.cmd.clone();

        let idx = self.executions.len();
        self.executions.push(TaskOutput::Waiting);
        self.running_threads.push((idx,
            thread::spawn(
                move || -> TaskOutput {
                    cmd.read()?.run()
                }
            )
        ));
    }

    pub fn update(&mut self) {
        let mut n = self.running_threads.len();
        let mut i = 0;

        while i < n {
            if self.running_threads[i].1.is_finished() {
                let (idx, handler) = self.running_threads.swap_remove(i);
                let res = self.update_log_and_stats(handler, idx);
                
                self.executions[idx] = res;
                n -= 1;
            } else {
                i += 1;
            }
        }
    }

    pub fn are_tasks_running(&self) -> bool {
        self.running_threads.len() > 0
    }

    pub fn iter(&self) -> core::slice::Iter<'_, TaskOutput> {
        self.executions.iter()
    }

    pub fn stats(&self) -> &TaskStatistic {
        &self.stats
    }
}