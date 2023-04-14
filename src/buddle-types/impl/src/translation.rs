use heck::{ToSnakeCase, ToUpperCamelCase};

// See https://doc.rust-lang.org/reference/keywords.html
static RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

/// Translates simple C++ type names to their Rust equivalents.
pub fn make_rust_type_name(ident: &str) -> String {
    // Types usually start with a "class "/"enum "/"union "/"struct " prefix,
    // so we need to remove that.
    let (_, ident) = ident.split_once(' ').unwrap();

    // Some KI types are nested which can't be directly replicated in Rust.
    // So we just remove the separator and go with the resulting ugly name
    // to ensure uniqueness.
    ident.replace("::", "").to_upper_camel_case()
}

fn cpp_type_to_rust_type_impl(name: &str) -> (Option<&'static str>, String) {
    match name {
        "bool" => (None, "bool".to_string()),

        "char" => (None, "i8".to_string()),
        "short" => (None, "i16".to_string()),
        "int" => (None, "i32".to_string()),
        "long" => (Some("LONG"), "i32".to_string()),

        "unsigned char" => (None, "u8".to_string()),
        "unsigned short" => (None, "u16".to_string()),
        "unsigned int" => (None, "u32".to_string()),
        "unsigned long" => (Some("ULONG"), "u32".to_string()),
        "unsigned __int64" => (None, "u64".to_string()),

        "float" => (None, "f32".to_string()),
        "double" => (None, "f64".to_string()),

        "std::string" | "char*" => (None, "RawString".to_string()),
        "std::wstring" | "wchar_t*" => (None, "RawWideString".to_string()),

        "bi2" => (None, "i2".to_string()),
        "bi3" => (None, "i3".to_string()),
        "bi4" => (None, "i4".to_string()),
        "bi5" => (None, "i5".to_string()),
        "bi6" => (None, "i6".to_string()),
        "bi7" => (None, "i7".to_string()),
        "s24" => (None, "i24".to_string()),

        "bui2" => (None, "u2".to_string()),
        "bui3" => (None, "u3".to_string()),
        "bui4" => (None, "u4".to_string()),
        "bui5" => (None, "u5".to_string()),
        "bui6" => (None, "u6".to_string()),
        "bui7" => (None, "u7".to_string()),
        "u24" => (None, "u24".to_string()),

        "class Vector3D" => (None, "Vec3".to_string()),
        "class Matrix3x3" => (None, "Mat3".to_string()),
        "class Quaternion" => (None, "Quat".to_string()),
        "class Euler" => (None, "Euler".to_string()),

        // TODO: Is this ok?
        "class SharedPointer" => (None, "SharedPtr".to_string()),

        s if s.ends_with('*') => {
            let mut new = make_rust_type_name(s.strip_suffix('*').unwrap());
            new.insert_str(0, "Ptr<");
            new.push('>');

            (None, new)
        }

        s => (None, make_rust_type_name(s)),
    }
}

/// A translated C++ type.
pub struct CppType<'a> {
    /// Type info override, if necessary.
    pub info: Option<&'a str>,
    /// The translated Rust type path.
    pub rust_ident: String,
}

/// Translates a given C++ type to its Rust equivalent.
pub fn cpp_type_to_rust_type(name: &str, container: bool) -> CppType<'_> {
    const DELIMITERS: &[char] = &['<', '>', ','];

    let mut rust_type = CppType {
        info: None,
        rust_ident: String::new(),
    };

    if container {
        rust_type.rust_ident.push_str("Vec<");
    }

    for s in name.split_inclusive(DELIMITERS) {
        // If this string is only a single identifier without type parameters,
        // we can take a shortcut here.
        if !s.ends_with(DELIMITERS) {
            let (info, ident) = cpp_type_to_rust_type_impl(s);
            rust_type.info = info;
            rust_type.rust_ident += &ident;

            break;
        }

        // Split the string into the current path to translate and its delimiter.
        let (path, delim) = s.split_at(s.len() - 1);
        let path = path.trim();

        if !path.is_empty() {
            let (info, ident) = cpp_type_to_rust_type_impl(path);
            if info.is_some() {
                // To ensure the info ends up being correct, we need to use
                // the real name of the whole type and not just the override
                // for the path we got here.
                // TODO: This is broken. Fix later.
                rust_type.info = Some(name);
            }
            rust_type.rust_ident += &ident;
        }

        rust_type.rust_ident.push_str(delim);
    }

    if container {
        rust_type.rust_ident.push('>');
    }

    rust_type
}

/// Translates a property's name to a Rust field identifier.
pub fn property_to_rust_field(ident: &str) -> String {
    // In rare cases, property names are nested paths into the structure
    // such as "m_gid.m_full". We want to lose the second part.
    let ident = ident.split('.').next().unwrap();

    // Translate from KI code style to Rust code style.
    // NOTE: Not every identifier starts with m_
    let ident = ident.strip_prefix("m_").unwrap_or(ident);
    let mut ident = ident.to_snake_case();

    // Escape the identifier if it happens to be a Rust keyword.
    if RUST_KEYWORDS.contains(&ident.as_str()) {
        ident.insert_str(0, "r#");
    }

    ident
}
