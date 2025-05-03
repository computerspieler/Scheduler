pub struct Command {
    command: String,
    arguments: Vec<String>,
    
    id: usize,
    
    on_sucess_stdout_save_path: Option<String>,
    on_sucess_stderr_save_path: Option<String>,
    on_failure_stdout_save_path: Option<String>,
    on_failure_stderr_save_path: Option<String>,
}

impl Command {
    pub fn run(&self) -> bool {
        let output = std::process::Command::new(&self.command)
            .args(&self.arguments)
            .output();
        match output {
        Ok(out) => {
            if let Some(path) = &self.on_sucess_stdout_save_path {
                //out.stdout
                
            }

            if let Some(path) = &self.on_sucess_stderr_save_path {

            }

            true
        }
        Err(e) => {
            false
        }
        }
    }
}