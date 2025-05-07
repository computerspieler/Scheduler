use std::{
	any::Any, collections::HashMap, convert::Infallible, io, ops::{ControlFlow, FromResidual, Try}, process::ExitStatus, sync::PoisonError, thread, time::{Duration, Instant}
};

#[derive(Debug)]
pub enum Log {
	Buffer(Vec<u8>),
	File(String),
	Nothing
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
    IOError(io::Error),
    ThreadError(Box<dyn Any + Send + 'static>),
	PoisonError
}

impl TaskOutput {
    pub fn title(&self) -> String {
        use TaskOutput::*;

        match self {
            NoError(_) => String::from("NoError"),
            IOError(e) => format!("IOError ({})", e.to_string()),
            ThreadError(_) => String::from("ThreadError"),
			PoisonError => String::from("PoisonError")
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
        _ => ControlFlow::Break(self)
        }
    }
}

impl FromResidual<TaskOutput> for TaskOutput {
    fn from_residual(res: TaskOutput) -> Self {
        use TaskOutput::*;

        match res {
        NoError(_) => unreachable!(),
        
        IOError(e) => IOError(e),
        ThreadError(e) => ThreadError(e),
		PoisonError => PoisonError
        }
    }
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
			stdout: Log::Buffer(output.stdout),
			stderr: Log::Buffer(output.stderr),
			start: start,
			duration: duration
		})
	}
}
