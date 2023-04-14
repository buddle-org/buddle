use std::{env, path::Path};

const TYPE_LIST_ENV: &str = "BUDDLE_TYPE_LIST";

fn main() -> anyhow::Result<()> {
    // Read the type list from an environment variable.
    let type_list = env::var(TYPE_LIST_ENV)
        .expect("`BUDDLE_TYPE_LIST` environment variable is not set for build");
    let type_list = Path::new(&type_list);

    let output = Path::new(&env::var("OUT_DIR")?)
        .join(type_list.file_name().unwrap())
        .with_extension("rs");
    let types = Path::new(&type_list);

    // Generate the types to the output file.
    buddle_types_impl::Builder::new(types, &output)?
        .ignore("PropertyClass")
        .finish()?;

    // Set an environment variable so `buddle-types` can find the file.
    println!(
        "cargo:rustc-env=BUDDLE_GENERATED_TYPES={}",
        output.display()
    );

    // Make sure the script re-runs after the type list file was changed.
    println!("cargo:rerun-if-env-changed={TYPE_LIST_ENV}");
    println!("cargo:rerun-if-changed={}", types.display());

    Ok(())
}
