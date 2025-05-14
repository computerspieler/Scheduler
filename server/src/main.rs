use std::{env, thread, time::Duration};

use log::{Level, LevelFilter, Metadata, Record};

use common::environment::Environment;

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
	log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
		.unwrap();

	let conf_path: Vec<String> = env::args().collect();
	if conf_path.len() != 2 {
		panic!("Usage: ./scheduler [PATH TO CONFIG]");
	}

	let mut env: Environment = serde_json::from_str(
		&String::from_utf8(
			std::fs::read(&conf_path[1])
				.unwrap()
		).unwrap()
		.as_str()
	).unwrap();

	dbg!(&env);
	loop {
		env.update();

		thread::sleep(Duration::from_millis(500));
	}
}
