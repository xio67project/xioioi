use pulldown_cmark::{Options, Parser, html};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};
use zip::ZipArchive;

use crate::db;

pub struct Example {
	pub input: String,
	pub output: String,
}

pub struct Pkg {
	pub doc: String,
	pub examples: Vec<Example>,
	pub groups: BTreeMap<String, Vec<String>>,
}

fn markdown(src: &str) -> String {
	let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
	let mut out = String::new();
	html::push_html(&mut out, Parser::new_ext(src, opts));
	out
}

fn prefix(names: &[String]) -> String {
	let files: Vec<&String> = names.iter().filter(|n| !n.ends_with('/')).collect();
	if files.iter().any(|n| *n == "config.toml") {
		return String::new();
	}
	let Some(first) = files.first() else {
		return String::new();
	};
	let top = first.split('/').next().unwrap_or("");
	if files.iter().all(|n| n.starts_with(&format!("{top}/"))) {
		format!("{top}/")
	} else {
		String::new()
	}
}

fn read(zip: &mut ZipArchive<File>, name: &str) -> io::Result<String> {
	let mut file = zip.by_name(name).map_err(io::Error::other)?;
	let mut out = String::new();
	file.read_to_string(&mut out)?;
	Ok(out)
}

pub fn load(id: &str) -> io::Result<Pkg> {
	let mut zip = ZipArchive::new(File::open(db::problem_zip(id))?).map_err(io::Error::other)?;
	let names: Vec<String> = zip.file_names().map(String::from).collect();
	let pre = prefix(&names);

	let doc = markdown(&read(&mut zip, &format!("{pre}DOC.md"))?);

	let in_dir = format!("{pre}in/");
	let mut tests: Vec<String> = names
		.iter()
		.filter_map(|n| n.strip_prefix(&in_dir)?.strip_suffix(".in"))
		.filter(|n| !n.contains('/'))
		.map(String::from)
		.collect();
	tests.sort();

	let mut examples = Vec::new();
	let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
	for name in tests {
		if name.starts_with('_') {
			let input = read(&mut zip, &format!("{pre}in/{name}.in"))?;
			let output = read(&mut zip, &format!("{pre}out/{name}.out"))?;
			examples.push(Example { input, output });
		} else {
			let group = name[..1].to_string();
			groups.entry(group).or_default().push(name);
		}
	}

	Ok(Pkg { doc, examples, groups })
}
