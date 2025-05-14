use std::{
	any::Any, collections::HashMap, convert::Infallible, io, ops::{ControlFlow, FromResidual, Try}, path::PathBuf, process::ExitStatus, sync::PoisonError, thread, time::{Duration, Instant}
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Log {
	Buffer(Vec<u8>),
	File(PathBuf),
	Nothing,
	Missing
}

impl Log {
	pub fn from_vec(buffer: Vec<u8>) -> Self {
		let n = buffer.len();
		if n > 0 {
			Log::Buffer(buffer)
		} else {
			Log::Nothing
		}
	}
}

#[derive(Debug)]
pub struct CommandOutcome {
	pub exit_status: ExitStatus,
	pub stdout: Log,
	pub stderr: Log,
	pub start: Instant,
	pub duration: Duration
}

impl CommandOutcome {
	pub fn is_success(&self) -> bool {
		self.exit_status.success()
	}
}

#[derive(Debug)]
pub enum TaskOutput {
    NoError(CommandOutcome),
	Waiting,
    IOError(io::Error),
    ThreadError(Box<dyn Any + Send + 'static>),
	PoisonError,
	TooManyThreadsError
}

impl TaskOutput {
    pub fn summary(&self) -> String {
        match self {
            TaskOutput::NoError(_) => String::from("NoError"),
			TaskOutput::Waiting => String::from("Waiting"),
            TaskOutput::IOError(e) => format!("IOError ({})", e.to_string()),
            TaskOutput::ThreadError(_) => String::from("ThreadError"),
			TaskOutput::PoisonError => String::from("PoisonError"),
			TaskOutput::TooManyThreadsError => String::from("TooManyThreadsError"),
        }
    }

	pub fn is_error(&self) -> bool {
        match self {
            TaskOutput::NoError(_) |
			TaskOutput::Waiting => false,

            TaskOutput::IOError(_) |
            TaskOutput::ThreadError(_) |
			TaskOutput::TooManyThreadsError |
			TaskOutput::PoisonError => true,
        }
	}
}

impl Try for TaskOutput {
    type Output = CommandOutcome;
    type Residual = TaskOutput;

    fn from_output(x: Self::Output) -> Self {
        TaskOutput::NoError(x)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
        TaskOutput::NoError(x) => ControlFlow::Continue(x),
		TaskOutput::Waiting |
		TaskOutput::IOError(_) |
		TaskOutput::ThreadError(_) |
		TaskOutput::TooManyThreadsError |
		TaskOutput::PoisonError => ControlFlow::Break(self)
        }
    }
}

impl FromResidual<TaskOutput> for TaskOutput {
    fn from_residual(res: TaskOutput) -> Self { res }
}

impl FromResidual<Result<Infallible, io::Error>> for TaskOutput {
    fn from_residual(res: Result<Infallible, io::Error>) -> Self {
        Self::IOError(res.unwrap_err())
    }
}

impl FromResidual<thread::Result<Infallible>> for TaskOutput {
    fn from_residual(res: thread::Result<Infallible>) -> Self {
        Self::ThreadError(res.unwrap_err())
    }
}

impl<T> FromResidual<Result<Infallible, PoisonError<T>>> for TaskOutput {
    fn from_residual(_: Result<Infallible, PoisonError<T>>) -> Self {
        Self::PoisonError
    }
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct Command {
    #[serde(rename = "program")]
	pub command: String,
    #[serde(rename = "args")]
	pub arguments: Vec<String>,
    #[serde(rename = "envs")]
	pub envs: Option<HashMap<String, String>>,
    #[serde(rename = "chdir")]
	#[serde(default = "default_path")]
	pub current_dir: PathBuf
}

fn default_path() -> PathBuf {
	std::env::current_dir().unwrap()
}

impl Command {
	pub fn run(&self) -> TaskOutput {
		let start = Instant::now();
		let mut cmd = std::process::Command::new(&self.command);

		cmd.args(&self.arguments);
		if let Some(map) = &self.envs {
			cmd.envs(map);
		}
		cmd.current_dir(&self.current_dir);

		let output = cmd.output()?;
		let duration = start.elapsed();

		TaskOutput::NoError(CommandOutcome {
			exit_status: output.status,
			stdout: Log::from_vec(output.stdout),
			stderr: Log::from_vec(output.stderr),
			start: start,
			duration: duration
		})
	}
}
