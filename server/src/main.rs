use std::{env, sync::{Arc, RwLock}, thread, time::Duration};

use log::LevelFilter;

use common::log::SimpleLogger;
use crate::environment::Environment;

mod environment;

pub static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
	log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
		.unwrap();

	let conf_path: Vec<String> = env::args().collect();
	if conf_path.len() != 2 {
		panic!("Usage: ./server [PATH TO CONFIG]");
	}

	let env: Arc<RwLock<Environment>> = Arc::new(RwLock::new(
		serde_json::from_str(
			&String::from_utf8(
				std::fs::read(&conf_path[1])
					.unwrap()
			).unwrap()
			.as_str()
		).unwrap()
	));
	
	dbg!(&env);
	loop {
		env.write().unwrap().update();
		thread::sleep(Duration::from_millis(500));
	}
}
