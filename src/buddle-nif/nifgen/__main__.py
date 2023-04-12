#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from pathlib import Path

from . import codegen

CRATE_ROOT = Path(__file__).parent.parent

NIF_XML = CRATE_ROOT / "nifxml" / "nif.xml"

BITFIELDS_RS = CRATE_ROOT / "src" / "bitfields.rs"
BITFLAGS_RS = CRATE_ROOT / "src" / "bitflags.rs"
COMPOUNDS_RS = CRATE_ROOT / "src" / "compounds.rs"
ENUMS_RS = CRATE_ROOT / "src" / "enums.rs"
OBJECTS_RS = CRATE_ROOT / "src" / "objects.rs"


def main():
    ctx = codegen.Context.parse(NIF_XML)

    # Emit all the bitfields first since those have no
    # dependencies and are the easiest.
    with open(BITFLAGS_RS, "w", encoding="utf-8") as f:
        f.write(ctx.get_header() + "\n")
        for bf in ctx.bitflags.values():
            codegen.emit_bitflags(f, bf)

    # Follow up with enums since those also have no
    # dependencies. They are leaned on C enums.
    with open(ENUMS_RS, "w", encoding="utf-8") as f:
        f.write(ctx.get_header() + "\n")
        for e in ctx.enums.values():
            codegen.emit_enum(f, e)

    # Bitfields are represented as Rust structs wrapping
    # integers. Some fields represent enum variants, so
    # we need to import those for proper type casting.
    with open(BITFIELDS_RS, "w", encoding="utf-8") as f:
        f.write(ctx.get_header())
        f.write("use crate::enums::*;\n\n")
        for bf in ctx.bitfields.values():
            codegen.emit_bitfield(f, bf)

    # Now we do compounds which may use the previously
    # generated types.
    # Some compounds hold references to NiObjects.
    with open(COMPOUNDS_RS, "w", encoding="utf-8") as f:
        f.write(ctx.get_header() + "\n")
        f.write("use crate::{bitflags::*, bitfields::*, enums::*, objects::*};\n\n")
        f.write("mod impls;\n\n")
        f.write("mod manual;\n")
        f.write("pub use self::manual::*;\n\n")
        for c in ctx.compounds.values():
            codegen.emit_compound(f, ctx, c)

    # Lastly, we generate the NiObjects.
    # Those may use any of the previously generated types.
    with open(OBJECTS_RS, "w", encoding="utf-8") as f:
        f.write(ctx.get_header())
        f.write("use binrw::{io::{Read, Seek}, BinResult, Error, ReadOptions};\n\n")
        f.write("use crate::{bitflags::*, bitfields::*, compounds::*, enums::*};\n\n")
        f.write("mod impls;\n\n")
        f.write("mod manual;\n")
        f.write("pub use self::manual::*;\n\n")

        codegen.emit_niobject_impl(f, ctx)

        # Now emit all the object types themselves.
        for obj in ctx.niobjects.values():
            codegen.emit_object(f, ctx, obj)

    print(f"Done! Make sure to validate basic types, {', '.join(codegen.MANUAL_COMPOUNDS)}!")

main()
