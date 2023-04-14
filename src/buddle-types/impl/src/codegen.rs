use std::{
    fs::File,
    io::{BufWriter, Write},
};

use crate::{
    translation::*,
    type_list::{Property, TypeDef},
};

/// Generates a given C++ type definition to the writer.
pub fn generate_struct(writer: &mut BufWriter<File>, type_def: &TypeDef) -> anyhow::Result<()> {
    writeln!(writer, "#[derive(Clone, Debug, Default, Type)]")?;
    writeln!(writer, "#[object(name = \"{}\")]", type_def.name)?;

    if !type_def.name.starts_with("class ") && !type_def.name.starts_with("struct ") {
        anyhow::bail!("trying to generate invalid type in struct context");
    }

    writeln!(
        writer,
        "pub struct {} {{",
        make_rust_type_name(&type_def.name)
    )?;
    for property in &type_def.properties {
        generate_property(writer, property)?;
    }
    writeln!(writer, "}}")?;

    Ok(())
}

fn generate_property(writer: &mut BufWriter<File>, property: &Property) -> anyhow::Result<()> {
    let rust_ty = cpp_type_to_rust_type(&property.r#type, property.dynamic);

    write!(
        writer,
        "#[property(name = \"{}\", flags({:?})",
        property.name, property.flags
    )?;
    if let Some(info) = rust_ty.info {
        write!(writer, ", info = {info}")?;
    }
    writeln!(writer, ")]")?;

    writeln!(
        writer,
        "pub {}: {},",
        property_to_rust_field(&property.name),
        rust_ty.rust_ident,
    )?;

    if !property.enum_options.is_empty() {
        todo!();
    }

    Ok(())
}
