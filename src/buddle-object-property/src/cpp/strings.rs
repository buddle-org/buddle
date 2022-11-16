use std::{
    fmt::{self, Write},
    string::{FromUtf16Error, FromUtf8Error},
};

/// A string type that stores its contents as raw bytes.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct RawString(
    /// The raw byte string.
    pub Vec<u8>,
);

impl From<&str> for RawString {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<String> for RawString {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<RawString> for Vec<u8> {
    fn from(s: RawString) -> Self {
        s.0
    }
}

impl TryFrom<RawString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: RawString) -> Result<Self, Self::Error> {
        String::from_utf8(value.0)
    }
}

impl core::ops::Deref for RawString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for RawString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for RawString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullString(\"")?;
        display_utf8(&self.0, f, str::escape_debug)?;
        write!(f, "\")")
    }
}

impl fmt::Display for RawString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf8(&self.0, f, str::chars)
    }
}

/// A wide string type that stores its contents as raw bytes.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct RawWideString(
    /// The raw byte string.
    pub Vec<u16>,
);

impl From<RawWideString> for Vec<u16> {
    fn from(s: RawWideString) -> Self {
        s.0
    }
}

impl From<&str> for RawWideString {
    fn from(s: &str) -> Self {
        Self(s.encode_utf16().collect())
    }
}

impl From<String> for RawWideString {
    fn from(s: String) -> Self {
        Self(s.encode_utf16().collect())
    }
}

impl TryFrom<RawWideString> for String {
    type Error = FromUtf16Error;

    fn try_from(value: RawWideString) -> Result<Self, Self::Error> {
        String::from_utf16(&value.0)
    }
}

impl core::ops::Deref for RawWideString {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for RawWideString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for RawWideString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf16(&self.0, f, core::iter::once)
    }
}

impl fmt::Debug for RawWideString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RawWideString(\"")?;
        display_utf16(&self.0, f, char::escape_debug)?;
        write!(f, "\")")
    }
}

fn display_utf16<Transformer: Fn(char) -> O, O: Iterator<Item = char>>(
    input: &[u16],
    f: &mut fmt::Formatter<'_>,
    t: Transformer,
) -> fmt::Result {
    char::decode_utf16(input.iter().copied())
        .flat_map(|r| t(r.unwrap_or(char::REPLACEMENT_CHARACTER)))
        .try_for_each(|c| f.write_char(c))
}

fn display_utf8<'a, Transformer: Fn(&'a str) -> O, O: Iterator<Item = char> + 'a>(
    mut input: &'a [u8],
    f: &mut fmt::Formatter<'_>,
    t: Transformer,
) -> fmt::Result {
    // Adapted from <https://doc.rust-lang.org/std/str/struct.Utf8Error.html>
    loop {
        match core::str::from_utf8(input) {
            Ok(valid) => {
                t(valid).try_for_each(|c| f.write_char(c))?;
                break;
            }
            Err(error) => {
                let (valid, after_valid) = input.split_at(error.valid_up_to());

                t(core::str::from_utf8(valid).unwrap()).try_for_each(|c| f.write_char(c))?;
                f.write_char(char::REPLACEMENT_CHARACTER)?;

                if let Some(invalid_sequence_length) = error.error_len() {
                    input = &after_valid[invalid_sequence_length..];
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}
