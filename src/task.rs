use std::{
    collections::LinkedList, fs, io, sync::{Arc, RwLock}, thread::{self, JoinHandle}, time::Duration
};

use crate::command::*;

#[derive(Debug, Default)]
pub struct TaskStatistic {
    average_duration: Duration
}

#[derive(Debug)]
pub struct Task {
    cmd: Arc<RwLock<Command>>,

    executions: LinkedList<TaskOutput>,
    running_threads: Vec<JoinHandle<TaskOutput>>,
    stats: TaskStatistic,

    log_path: Option<String>
}

impl Task {
    pub fn new(cmd: Command) -> Self {
        Self {
            cmd: Arc::new(RwLock::new(cmd)),

            executions: LinkedList::new(),
            running_threads: Vec::new(),
            stats: TaskStatistic::default(),

            log_path: Some(String::from("logs/"))
        }
    }

    fn update_log_and_stats(&mut self, i: usize) -> TaskOutput {
        let n = self.executions.len();

        let mut res = self.running_threads.swap_remove(i).join()??;

        if let Some(path) = &self.log_path {
            if let Log::Buffer(log) = &res.stdout {
                let path = format!("{}/{}.out", path, n);
                fs::write(&path, log)?;
                res.stdout = Log::File(path);
            }
            if let Log::Buffer(log) = &res.stderr {
                let path = format!("{}/{}.err", path, n);
                fs::write(&path, log)?;
                res.stderr = Log::File(path);
            }
        }
        
        let n: u32 = n.try_into().unwrap();
        let n: f64 = n.try_into().unwrap();
        
        self.stats.average_duration =
            self.stats.average_duration.mul_f64(n / (n + 1.)) +
            res.duration.div_f64(n + 1.);

        TaskOutput::NoError(res)
    }

    pub fn run(&mut self)
        where Arc<RwLock<Command>>: Sync
    {
        let cmd = self.cmd.clone();

        self.running_threads.push(thread::spawn(
            move || -> TaskOutput {
                cmd.read()?.run()
            }
        ));
    }

    pub fn update(&mut self) {
        let mut n = self.running_threads.len();
        let mut i = 0;

        while i < n {
            if self.running_threads[i].is_finished() {
                let out = self.update_log_and_stats(i);
                self.executions.push_back(out);
                n -= 1;
            } else {
                i += 1;
            }
        }
    }

    pub fn are_tasks_running(&self) -> bool {
        self.running_threads.len() > 0
    }

    pub fn iter(&self) -> std::collections::linked_list::Iter<'_, TaskOutput> {
        self.executions.iter()
    }
}