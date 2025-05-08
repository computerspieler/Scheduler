use std::{
	any::Any, collections::HashMap, convert::Infallible, io, ops::{ControlFlow, FromResidual, Try}, process::ExitStatus, sync::PoisonError, thread, time::{Duration, Instant}
};

#[derive(Debug)]
pub enum Log {
	Buffer(Vec<u8>),
	File(String),
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
	PoisonError
}

impl TaskOutput {
    pub fn title(&self) -> String {
        match self {
            TaskOutput::NoError(_) => String::from("NoError"),
			TaskOutput::Waiting => String::from("Waiting"),
            TaskOutput::IOError(e) => format!("IOError ({})", e.to_string()),
            TaskOutput::ThreadError(_) => String::from("ThreadError"),
			TaskOutput::PoisonError => String::from("PoisonError"),
        }
    }

	pub fn is_error(&self) -> bool {
        match self {
            TaskOutput::NoError(_) |
			TaskOutput::Waiting => false,

            TaskOutput::IOError(_) |
            TaskOutput::ThreadError(_) |
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

#[derive(Debug)]
pub struct Command {
	command: String,
	arguments: Vec<String>,
	envs: HashMap<String, String>
}

impl Command {
	pub fn new<CS: ToString, AS: ToString>(command: CS, arguments: Vec<AS>) -> Self {
		Self {
			command: command.to_string(),
			arguments: arguments.into_iter()
				.map(|x| x.to_string())
				.collect(),
			envs: HashMap::new()
		}
	}

	pub fn run(&self) -> TaskOutput {
		let start = Instant::now();
		let output = std::process::Command::new(&self.command)
			.args(&self.arguments)
			.envs(&self.envs)
			.output()?;
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
