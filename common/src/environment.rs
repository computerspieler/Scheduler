use std::{io::{ErrorKind, Read}, net::{SocketAddr, TcpListener, TcpStream}, path::PathBuf};

use log::{error, info};
use serde::{de::Deserializer, Deserialize};
use serde_json;

use crate::group::TaskGroup;

#[derive(Debug)]
pub struct Environment {
    groups: Vec<TaskGroup>,
    log: Option<PathBuf>,
    listener: TcpListener
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        pub struct EnvironmentJson {
            log: Option<PathBuf>,
            groups: Vec<TaskGroup>
        }

        let val = EnvironmentJson::deserialize(deserializer)?;
        let mut output = Environment {
            groups: val.groups,
            log: None,
            //TODO: Set the listener based on config
	        listener: TcpListener::bind("127.0.0.1:65533").unwrap()
        };
        output.listener.set_nonblocking(true).unwrap();

        if let Some(path) = val.log {
            output.set_log_path(path);
        }

        Ok(output)
    }
}

impl Environment {
    fn add_new_group(&mut self, mut task_group: TaskGroup) {
        let id = self.groups.len();

        if let Some(path) = &self.log {
            let group_path = Self::get_task_group_log_path(&path, id);
            task_group.set_log_path(group_path);
        }

        self.groups.push(task_group)
    }

    fn on_new_stream(&mut self, mut stream: TcpStream, addr: SocketAddr) {
        info!("[ENV] New connection from {}", addr);

        let mut buf = String::new();
        match stream.read_to_string(&mut buf) {
        Ok(_) => {
            match serde_json::from_str(buf.as_str()) {
            Ok(tgroup) => self.add_new_group(tgroup),
            Err(e) => error!("[ENV] Error while parsing data: {}", e)
            }
        },
        Err(e) => 
            error!("[ENV] Error while retrieving data: {}", e)
        }
    }

    pub fn update(&mut self) {
        info!("[ENV] Checking for new updates");
        match self.listener.accept() {
        Ok((stream, addr)) => self.on_new_stream(stream, addr),
        Err(e) if e.kind() == ErrorKind::WouldBlock => {}
        Err(e) => error!("[ENV] Network Error: {}", e)
        }

        info!("[ENV] Update");
        for group in self.groups.iter_mut() {
            group.update();
        }
    }

    fn get_task_group_log_path(path: &PathBuf, id: usize) -> PathBuf{
        path.join(id.to_string())
    }

    fn set_log_path(&mut self, path: PathBuf) {
        if !path.exists() {
            std::fs::create_dir(&path).unwrap();
        }

        for (id, group) in self.groups.iter_mut().enumerate() {
            let group_path = Self::get_task_group_log_path(&path, id);
            group.set_log_path(group_path);
        }
        self.log = Some(path);
    }
}