mod task;

fn main() {
	dbg!(
		std::process::Command::new("lsd")
            .arg("-la")
            .output()
	);
}
