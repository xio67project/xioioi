use std::process::Command;

const WEB: &str = "src/frontend/web";

fn main() {
	println!("cargo:rerun-if-changed={WEB}/tailwind.css");
	println!("cargo:rerun-if-changed={WEB}/templates");
	println!("cargo:rerun-if-changed={WEB}/static/js");

	let input = format!("{WEB}/tailwind.css");
	let output = format!("{WEB}/static/css/app.css");
	let status = Command::new("tailwindcss")
		.args(["-i", &input, "-o", &output, "--minify"])
		.status();

	match status {
		Ok(s) if s.success() => {}
		Ok(s) => panic!("tailwindcss failed: {s}"),
		Err(e) => println!("cargo:warning=tailwindcss not found, skipping css: {e}"),
	}
}
