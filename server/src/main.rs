use std::{env, io::{self, Error, ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, path::PathBuf, sync::{Arc, RwLock}, thread, time::Duration};

use log::{error, info, LevelFilter};

use common::{group::TaskGroup, log::SimpleLogger, queries::Queries};
use serde::{Deserialize, Deserializer};
use crate::environment::Environment;

mod environment;

pub static LOGGER: SimpleLogger = SimpleLogger;

pub struct Server {
    env: Arc<RwLock<Environment>>,
    listener: Option<TcpListener>
}

impl<'de> Deserialize<'de> for Server {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        pub struct EnvironmentJson {
            log: Option<PathBuf>,
            listening: Option<String>,
            groups: Vec<TaskGroup>
        }

        let val = EnvironmentJson::deserialize(deserializer)?;
        let mut output_env = Environment {
            groups: val.groups,
            log: None,
			dirty: false
        };
        if let Some(path) = val.log {
            output_env.set_log_path(path);
        }

		let listener = val.listening
			.map(|addr| {
				let out = TcpListener::bind(&addr).expect("Unable to connect");
				info!("Sucessfully connected to {}", addr);
				out
			});

        Ok(Server {
			env: Arc::new(RwLock::new(output_env)),
			listener: listener
		})
    }
}

fn query_handler(query: Queries, stream: &mut TcpStream, env: Arc<RwLock<Environment>>) -> io::Result<()> {
	match query {
	Queries::Ok => Ok(()),
	Queries::NewTaskGroup(stg) => {
		let mut env = env.write()
			.expect("Unable to write to env");
		env.add_new_group(TaskGroup::from(stg));
		stream.write_all(
			serde_json::to_vec(
				&Queries::Ok
			).unwrap()
			.as_slice()
		)
	}
	}
}

fn network_handler(listener: TcpListener, env: Arc<RwLock<Environment>>) {
	thread::spawn(move || {
		for stream in listener.incoming() {
			let env = env.clone();
			thread::spawn(move || -> std::io::Result<()> {
				let mut stream = stream?;

				let mut buf = String::new();
				stream.read_to_string(&mut buf)?;
				
				match serde_json::from_str::<Queries>(buf.as_str()) {
				Ok(query) => query_handler(query, &mut stream, env),
				Err(e) => {
					error!("[ENV] Error while parsing data: {}", e);
					Err(Error::from(ErrorKind::InvalidData))
				}
				}
			});
		}
	});
}

fn main() {
	log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
		.unwrap();

	let conf_path: Vec<String> = env::args().collect();
	if conf_path.len() != 2 {
		panic!("Usage: ./server [PATH TO CONFIG]");
	}

	let server: Server = serde_json::from_str(
			&String::from_utf8(
				std::fs::read(&conf_path[1])
					.unwrap()
			).unwrap()
			.as_str()
		).unwrap();

	if let Some(listener) = server.listener {
		let env = server.env.clone();
		network_handler(listener, env);
	}
	
	loop {
		server.env.write().unwrap().update();
		thread::sleep(Duration::from_millis(500));
	}
}
