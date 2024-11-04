use std::path::PathBuf;

use crux_core::typegen::TypeGen;
use shared::App;

fn main() -> anyhow::Result<()> {
	println!("cargo:rerun-if-changed=../shared");
	let output_root = PathBuf::from("./generated");

	let mut gen = TypeGen::new();
	gen.register_app::<App>()?;
	// gen.register_type::<HttpError>()?;

	// gen.swift("SharedTypes", output_root.join("swift"))?;
	gen.java("dev.m00nlit.cruxim.shared_types", output_root.join("java"))?;
	// gen.typescript("shared_types", output_root.join("typescript"))?;

	Ok(())
}
