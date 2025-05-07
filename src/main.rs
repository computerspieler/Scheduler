#![feature(try_trait_v2)]

use std::{thread::sleep, time::Duration};

mod command;
mod task;

fn main() {
	let mut task = task::Task::new(
		command::Command::new(
			"/bin/sleep",
			vec!["5"]
		)
	);

	let _ = std::fs::remove_dir_all("logs");
	let _ = std::fs::create_dir("logs");

	for _ in 0 .. 1000 {
		task.run();
	}

	while task.are_tasks_running() {
		dbg!(task.update());
		sleep(Duration::from_secs(1));
		println!("Waiting");
	}

	for (i, exec) in task.iter().enumerate() {
		println!("Execution {}: {}", i, exec.title())
	}
	dbg!(task);
}
