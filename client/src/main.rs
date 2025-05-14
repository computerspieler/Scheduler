use std::{env, io::{self, Write}, net::TcpStream};

use log::LevelFilter;

use common::{
	group::SerializedTaskGroup, log::SimpleLogger, queries::Queries
};

pub static LOGGER: SimpleLogger = SimpleLogger;

fn main() -> io::Result<()> {
	log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
		.unwrap();

    let conf_path: Vec<String> = env::args().collect();
    if conf_path.len() != 2 {
        panic!("Usage: ./client [PATH TO CONFIG]");
    }

    let task_group: SerializedTaskGroup = serde_json::from_str(
        &String::from_utf8(
            std::fs::read(&conf_path[1])
                .unwrap()
        ).unwrap()
        .as_str()
    ).unwrap();

    let formatted_conf =
        serde_json::to_string(
            &Queries::NewTaskGroup(task_group)
        ).unwrap();
    
    let mut stream = TcpStream::connect("127.0.0.1:65533")?;
    stream.write(formatted_conf.as_bytes())?;
    Ok(())
}
