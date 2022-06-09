use clap::IntoApp;
use clap_complete::{
	generate_to,
	shells::{Bash, Fish, Zsh},
};
use std::path::PathBuf;

include!("src/args.rs");

fn main() {
	let outdir = PathBuf::from("../target");
	let mut cmd = CliArgs::command();
	let path = generate_to(Bash, &mut cmd, "pop-snapshot", &outdir)
		.expect("failed to generate bash completions");
	println!("cargo:warning=bash completion file generated: {:?}", path);
	let path = generate_to(Zsh, &mut cmd, "pop-snapshot", &outdir)
		.expect("failed to generate zsh completions");
	println!("cargo:warning=zsh completion file generated: {:?}", path);
	let path = generate_to(Fish, &mut cmd, "pop-snapshot", outdir)
		.expect("failed to generate fish completions");
	println!("cargo:warning=fish completion file generated: {:?}", path);
}
