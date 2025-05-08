#![feature(try_trait_v2)]
#![allow(dead_code)]

use std::{thread::sleep, time::Duration};

use log::{info, Record, Level, Metadata, LevelFilter};

mod command;
mod task;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
	let mut task = task::Task::new(task::TaskConfig {
		cmd: command::Command::new(
			"/bin/sleep",
			vec!["5"]
		),
		log_path: Some(String::from("logs/")),
		max_concurrent_iteration: Some(700)
	});

	let _ = std::fs::remove_dir_all("logs");
	let _ = std::fs::create_dir("logs");

	log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
		.unwrap();
	
	for _ in 0 .. 300 {
		task.run();
	}

	for nb_iterations in 0 .. {
		if task.nb_running_tasks() == 0 {
			break;
		}

		if task.update() {
			sleep(Duration::from_millis(50));
		} else {
			if nb_iterations < 20 {
				for _ in 0 .. 100 {
					task.run();
				}
			}
			sleep(Duration::from_secs(1));
		}
		info!("Waiting, {} running threads", task.nb_running_tasks());
	}

	info!("{}", task.stats());
}
