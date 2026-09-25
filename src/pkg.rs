use markdown::{CompileOptions, Constructs, Options, ParseOptions};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
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

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME: LazyLock<Theme> = LazyLock::new(|| ThemeSet::load_defaults().themes["base16-ocean.dark"].clone());

fn markdown(src: &str) -> String {
	let opts = Options {
		parse: ParseOptions {
			constructs: Constructs {
				html_flow: false,
				html_text: false,
				..Constructs::gfm()
			},
			..ParseOptions::gfm()
		},
		compile: CompileOptions::gfm(),
	};
	let html = markdown::to_html_with_options(src, &opts).unwrap_or_default();
	highlight_blocks(&html)
}

fn unescape(s: &str) -> String {
	s.replace("&lt;", "<")
		.replace("&gt;", ">")
		.replace("&quot;", "\"")
		.replace("&#x27;", "'")
		.replace("&amp;", "&")
}

fn highlight(code: &str, lang: &str) -> Option<String> {
	let syntax = SYNTAXES
		.find_syntax_by_token(lang)
		.or_else(|| SYNTAXES.syntaxes().iter().find(|s| s.name.eq_ignore_ascii_case(lang)))?;
	let mut lines = HighlightLines::new(syntax, &THEME);
	let mut out = String::new();
	for line in LinesWithEndings::from(code) {
		let ranges = lines.highlight_line(line, &SYNTAXES).ok()?;
		out.push_str(&styled_line_to_highlighted_html(&ranges, IncludeBackground::No).ok()?);
	}
	Some(out)
}

fn highlight_blocks(html: &str) -> String {
	const OPEN: &str = "<pre><code class=\"language-";
	const CLOSE: &str = "</code></pre>";
	let mut out = String::new();
	let mut rest = html;

	while let Some(start) = rest.find(OPEN) {
		out.push_str(&rest[..start]);
		let after = &rest[start + OPEN.len()..];
		let (Some(quote), Some(end)) = (after.find("\">"), after.find(CLOSE)) else {
			out.push_str(&rest[start..]);
			return out;
		};
		let lang = &after[..quote];
		let body = &after[quote + 2..end];
		match highlight(&unescape(body), lang) {
			Some(colored) => out.push_str(&format!("{OPEN}{lang}\">{colored}{CLOSE}")),
			None => out.push_str(&rest[start..start + OPEN.len() + end + CLOSE.len()]),
		}
		rest = &after[end + CLOSE.len()..];
	}

	out.push_str(rest);
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
