# -*- coding: utf-8 -*-

from .predicates import *

_BUILTIN_TYPES = {
    "uint64": "u64",
    "int64": "i64",
    "ulittle32": "u32",
    "uint": "u32",
    "int": "i32",
    "ushort": "u16",
    "short": "i16",
    "char": "i8",
    "byte": "u8",
    "bool": "bool",
    "float": "f32",
    "hfloat": "f16",
    "string": "NiString",
    "#T#": "T",
}

_RUST_KEYWORDS = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
    "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
]


def _convert_struct_name(name):
    return name[0].upper() + name[1:]


def convert_field_name(name):
    if name == "#ARG#":
        return "arg"

    if "." in name:
        # Likely a version.
        s = name.split(".")
        return f"FileVersion({', '.join(s)})"

    name = name.lower().replace(" ", "_").replace(":", "")
    if name in _RUST_KEYWORDS:
        name = f"r#{name}"
    return name


def _convert_variant_name(name):
    return name.replace(" ", "").replace(":", "")


def _convert_type(ty):
    if ty is None:
        return None

    return _BUILTIN_TYPES.get(ty, _convert_struct_name(ty))


def _format_doc(doc):
    result = ""
    if doc:
        doc = doc.strip()
        lines = [line.lstrip().rstrip() for line in doc.splitlines()]
        for line in lines:
            result += f"/// {line}\n"
    return result


class Type:
    def __init__(self, ty, t_arg, arr1, arr2, arg, cond, ver1, ver2):
        self.ty = ty
        self.t_arg = t_arg
        self.arr1 = arr1
        self.arr2 = arr2
        self.arg = arg
        self.cond = cond
        self.ver1 = ver1
        self.ver2 = ver2

    def get_inner_type(self):
        ty = self.ty
        if self.t_arg:
            ty += f"<{self.t_arg}>"
        return ty

    def get_count_attr(self):
        from .expr import parse_expression

        # For Vecs, we need to return a count attribute
        # so that BinRead knows how many elements we want.
        if self.arr1:
            if not self.arr1.isnumeric():
                calc = parse_expression(self.arr1)
                return f"#[br(count = {calc})]\n"

        return ""

    def get_parse_with_attr(self):
        # bools are not supported out of the box due to
        # not having standard representation.
        if self.ty == "bool":
            return "#[br(map = |b: u8| b != 0)]\n"

        return ""

    def get_vercond_attr(self):
        stmt = ""

        if self.ver1 is not None:
            stmt = f"FileVersion({self.ver1.replace('.', ',')}) <= _header_version"
        if self.ver2 is not None:
            if stmt != "":
                stmt += " && "
            stmt += f"_header_version <= FileVersion({self.ver2.replace('.', ',')})"

        return stmt

    def get_full_type(self):
        ty = self.get_inner_type()
        if self.arr1:
            if self.arr1.isnumeric():
                if self.arr2 and self.arr2.isnumeric():
                    size = f"{self.arr1} + {self.arr2}"
                else:
                    size = f"{self.arr1}"
                ty = f"[{ty}; {size}]"
            else:
                ty = f"Vec<{ty}>"
        return ty

    def is_optional(self):
        return self.cond is not None

    def get_arg(self):
        if self.arg is not None:
            if self.arg.isnumeric():
                return self.arg
            else:
                return convert_field_name(self.arg)

        return None

    def __str__(self):
        ty = self.get_full_type()
        if self.is_optional():
            ty = f"Option<{ty}>"
        return ty

    @classmethod
    def from_xml(cls, attrib):
        ty = _convert_type(attrib["type"])
        t_arg = _convert_type(attrib.get("template"))
        arr1 = attrib.get("arr1")
        arr2 = attrib.get("arr2")
        arg = attrib.get("arg")
        cond = attrib.get("cond")
        ver1 = attrib.get("ver1")
        ver2 = attrib.get("ver2")

        return cls(ty, t_arg, arr1, arr2, arg, cond, ver1, ver2)

    def matches(self, other):
        return (
            self.ty == other.ty
            and self.t_arg == other.t_arg
            and self.arr1 == other.arr1
            and self.arr2 == other.arr2
            and self.arg == other.arg
        )


class BitFlags:
    def __init__(self, name, doc, storage):
        self.name = name
        self.doc = doc
        self.storage = storage

        self.flags = {}

    @classmethod
    def from_xml(cls, entry):
        obj = cls(
            entry.attrib["name"],
            _format_doc(entry.text),
            _convert_type(entry.attrib["storage"])
        )

        for flag in filter(lambda f: should_emit_member(f.attrib), entry):
            bit = int(flag.attrib["bit"])

            name = _convert_variant_name(flag.attrib["name"])
            if "prefix" in entry.attrib:
                name = entry.attrib["prefix"] + "_" + name

            obj.flags[name] = bit

        return obj


class Enum:
    def __init__(self, name, doc, storage):
        self.name = name
        self.doc = doc
        self.storage = storage

        self.options = {}

    @classmethod
    def from_xml(cls, entry):
        obj = cls(
            entry.attrib["name"],
            _format_doc(entry.text),
            _convert_type(entry.attrib["storage"]),
        )

        for option in filter(lambda o: should_emit_member(o.attrib), entry):
            value = option.attrib["value"]
            if value.startswith("0x"):
                value = int(value, 16)
            else:
                value = int(value)

            variant_name = _convert_variant_name(option.attrib["name"])
            if "prefix" in entry.attrib:
                variant_name = entry.attrib["prefix"] + "_" + variant_name

            obj.options[variant_name] = value

        return obj


class BitField:
    def __init__(self, name, doc, storage):
        self.name = name
        self.doc = doc
        self.storage = storage

        self.fields = {}

    @classmethod
    def from_xml(cls, entry, prefixes):
        obj = cls(
            entry.attrib["name"],
            _format_doc(entry.text),
            _convert_type(entry.attrib["storage"]),
        )

        for field in filter(lambda f: should_emit_member(f.attrib), entry):
            pos = field.attrib["pos"]
            mask = field.attrib["mask"]
            name = convert_field_name(field.attrib["name"])
            ty = Type.from_xml(field.attrib)

            default = field.attrib.get("default")
            if default is not None and ty.ty in prefixes:
                default = prefixes[ty.ty] + "_" + default

            obj.fields[name] = (ty, pos, mask, default)

        return obj


class Compound:
    def __init__(self, name, doc, generic):
        self.name = name
        self.doc = doc
        self.generic = generic

        self.fields = None

    def get_import_attr(self):
        field_needs_arg = any("#ARG#" in f.cond if f.cond else False for f in self.fields.values())

        if self.name in ("Morph", "NiAGDDataBlocks") or field_needs_arg:
            return f"#[br(import(arg: usize, _header_version: FileVersion))]\n"
        return "#[br(import(_header_version: FileVersion))]\n"

    @classmethod
    def from_xml(cls, entry):
        obj = cls(
            _convert_type(entry.attrib["name"]),
            _format_doc(entry.text),
            entry.attrib.get("generic", False) == "true",
        )

        obj.fields = {}
        for field in filter(lambda f: should_emit_member(f.attrib), entry):
            name = convert_field_name(field.attrib["name"])
            if name in obj.fields:
                continue

            obj.fields[name] = Type.from_xml(field.attrib)

        return obj


class NiObject:
    def __init__(self, name, doc, parent):
        self.name = name
        self.doc = doc
        self.parent = parent

        self.fields = None

    @classmethod
    def from_xml(cls, entry):
        obj = NiObject(
            _convert_struct_name(entry.attrib["name"]),
            _format_doc(entry.text),
            entry.attrib["inherit"],
        )

        obj.fields = {}
        for field in filter(lambda f: should_emit_member(f.attrib), entry):
            name = convert_field_name(field.attrib["name"])
            field = Type.from_xml(field.attrib)

            if name in obj.fields:
                old = obj.fields[name]

                if not field.cond and not old.cond:
                    continue

                if not field.matches(old) and (
                    obj.name != "NiPalette" or name != "palette"
                ):
                    print(
                        f"Redeclaration of field {name} in {obj.name}, with incompatible signature:"
                    )
                    print(f" old: {old}")
                    print(f" new: {field}")
                else:
                    obj.fields[name].cond = f"({old.cond}) #OR# ({field.cond})"

            else:
                if obj.name == "NiPalette" and name == "palette":
                    field.arr1 = "Num Entries"
                    field.cond = None

                obj.fields[name] = field

        return obj
