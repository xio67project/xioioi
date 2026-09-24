use std::env;
use std::fs;
use std::process::Command;

const WEB: &str = "src/frontend/web";

fn run(name: &str, args: &[&str]) {
	let status = Command::new(name).args(args).status();
	match status {
		Ok(s) if s.success() => {}
		Ok(s) => panic!("{name} failed: {s}"),
		Err(e) if env::var("CI").is_ok() => panic!("{name} not found: {e}"),
		Err(e) => println!("cargo:warning={name} not found, skipping: {e}"),
	}
}

fn tailwind() {
	let input = format!("{WEB}/tailwind.css");
	let output = format!("{WEB}/static/css/app.css");
	run("tailwindcss", &["-i", &input, "-o", &output, "--minify"]);
}

fn typecheck() {
	let project = format!("{WEB}/tsconfig.json");
	run("tsc", &["-p", &project]);
}

fn typescript() {
	let mut files = Vec::new();
	for entry in fs::read_dir(format!("{WEB}/ts")).unwrap() {
		let path = entry.unwrap().path();
		if path.extension().is_some_and(|e| e == "ts") {
			files.push(path.to_string_lossy().into_owned());
		}
	}
	let outdir = format!("--outdir={WEB}/static/js");
	let mut args: Vec<&str> = files.iter().map(String::as_str).collect();
	args.push(&outdir);
	args.push("--minify");
	run("esbuild", &args);
}

fn main() {
	println!("cargo:rerun-if-changed={WEB}/tailwind.css");
	println!("cargo:rerun-if-changed={WEB}/templates");
	println!("cargo:rerun-if-changed={WEB}/ts");
	println!("cargo:rerun-if-changed={WEB}/tsconfig.json");

	tailwind();
	typecheck();
	typescript();
}
