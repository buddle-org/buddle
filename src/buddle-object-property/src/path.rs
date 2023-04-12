//! Recursive access to nested values in reflected types.

use std::{iter::Peekable, str::Chars};

use crate::{Type, TypeMut, TypeRef};

/// Dynamic access into nested data structures by paths.
///
/// Paths are strings in the form of `"container[1].my_element"`
/// to access the `my_element` property of the object stored
/// at index `1` of container `my_container`.
///
/// In that spirit, path strings try to approximate actual
/// Rust syntax to resolve a value as closely as possible.
pub trait PathAccess: Type {
    /// Attempts to resolve a given path into this value
    /// and returns an immutable reference on success.
    fn path(&self, path: &str) -> anyhow::Result<&dyn Type>;

    /// Attempts to resolve a given path into this value
    /// and returns a mutable reference on success.
    fn path_mut(&mut self, path: &str) -> anyhow::Result<&mut dyn Type>;

    /// Resolves a path with [`PathAccess::path`] and attempts
    /// to downcast into a concrete type.
    fn path_as<T: Type>(&self, path: &str) -> anyhow::Result<&T> {
        self.path(path).and_then(|p| {
            p.downcast_ref()
                .ok_or_else(|| anyhow::anyhow!("cannot downcast path element into incorrect type"))
        })
    }

    /// Resolves a path with [`PathAccess::path_mut`] and
    /// attempts to downcast into a concrete type.
    fn path_as_mut<T: Type>(&mut self, path: &str) -> anyhow::Result<&mut T> {
        self.path_mut(path).and_then(|p| {
            p.downcast_mut()
                .ok_or_else(|| anyhow::anyhow!("cannot downcast path element into incorrect type"))
        })
    }
}

impl PathAccess for dyn Type {
    fn path(&self, path: &str) -> anyhow::Result<&dyn Type> {
        let mut start = true;
        let mut current = self;
        let mut depth = 0;
        let mut lexer = Lexer::new(path);

        while let Some(token) = lexer.next() {
            match token {
                Token::Dot => {
                    if let Some(Token::Ident(ident)) = lexer.next() {
                        current = access_field(current, ident, depth)?;
                        depth += 1;
                    } else {
                        anyhow::bail!("'.' must be followed by an identifier at {:?}", lexer.pos);
                    }
                }

                Token::LBracket => {
                    if let Some(Token::Ident(ident)) = lexer.next() {
                        current = access_container(current, ident, depth)?;
                    } else {
                        anyhow::bail!("'[' must be followed by an identifier at {:?}", lexer.pos);
                    }

                    if lexer.next() != Some(Token::RBracket) {
                        anyhow::bail!("expected token ']' at {:?}", lexer.pos);
                    }

                    depth += 1;
                }

                Token::RBracket => {
                    anyhow::bail!("unexpected token ']' encountered at {:?}", lexer.pos);
                }

                Token::Ident(ident) => {
                    if start {
                        current = access_field(current, ident, depth)?;
                        depth += 1;
                    } else {
                        anyhow::bail!("expected '.' or '[' before ident at {:?}", lexer.pos);
                    }
                }

                // The iterator ends when this token occurs. We will never see it here.
                Token::End => unreachable!(),
            }

            // After the first run, we're not at the start anymore.
            start = false;
        }

        Ok(current)
    }

    fn path_mut(&mut self, path: &str) -> anyhow::Result<&mut dyn Type> {
        let mut start = true;
        let mut current = self;
        let mut depth = 0;
        let mut lexer = Lexer::new(path);

        while let Some(token) = lexer.next() {
            match token {
                Token::Dot => {
                    if let Some(Token::Ident(ident)) = lexer.next() {
                        current = access_field_mut(current, ident, depth)?;
                        depth += 1;
                    } else {
                        anyhow::bail!("'.' must be followed by an identifier at {:?}", lexer.pos);
                    }
                }

                Token::LBracket => {
                    if let Some(Token::Ident(ident)) = lexer.next() {
                        current = access_container_mut(current, ident, depth)?;
                    } else {
                        anyhow::bail!("'[' must be followed by an identifier at {:?}", lexer.pos);
                    }

                    if lexer.next() != Some(Token::RBracket) {
                        anyhow::bail!("expected token ']' at {:?}", lexer.pos);
                    }

                    depth += 1;
                }

                Token::RBracket => {
                    anyhow::bail!("unexpected token ']' encountered at {:?}", lexer.pos);
                }

                Token::Ident(ident) => {
                    if start {
                        current = access_field_mut(current, ident, depth)?;
                        depth += 1;
                    } else {
                        anyhow::bail!("expected '.' or '[' before ident at {:?}", lexer.pos);
                    }
                }

                // The iterator ends when this token occurs. We will never see it here.
                Token::End => unreachable!(),
            }

            // After the first run, we're not at the start anymore.
            start = false;
        }

        Ok(current)
    }
}

impl<T: Type> PathAccess for T {
    fn path(&self, path: &str) -> anyhow::Result<&dyn Type> {
        <dyn Type>::path(self, path)
    }

    fn path_mut(&mut self, path: &str) -> anyhow::Result<&mut dyn Type> {
        <dyn Type>::path_mut(self, path)
    }
}

fn access_field<'t>(
    value: &'t dyn Type,
    ident: &str,
    depth: usize,
) -> anyhow::Result<&'t dyn Type> {
    let cls = match value.type_ref() {
        TypeRef::Class(value) => value,
        _ => anyhow::bail!("expected structure at depth {depth:?}"),
    };

    let list = cls.property_list();
    list.property(ident)
        .map(|view| cls.property(view))
        .ok_or_else(|| {
            anyhow::anyhow!("value at depth {depth:?} does not have a field named {ident}")
        })
}

fn access_field_mut<'t>(
    value: &'t mut dyn Type,
    ident: &str,
    depth: usize,
) -> anyhow::Result<&'t mut dyn Type> {
    let cls = match value.type_mut() {
        TypeMut::Class(value) => value,
        _ => anyhow::bail!("expected structure at depth {depth:?}"),
    };

    let list = cls.property_list();
    list.property(ident)
        .map(|view| cls.property_mut(view))
        .ok_or_else(|| {
            anyhow::anyhow!("value at depth {depth:?} does not have a field named {ident}")
        })
}

fn access_container<'t>(
    value: &'t dyn Type,
    index: &str,
    depth: usize,
) -> anyhow::Result<&'t dyn Type> {
    let container = match value.type_ref() {
        TypeRef::Container(value) => value,
        _ => anyhow::bail!("expected container to index into at depth {depth:?}"),
    };

    let index = index.parse()?;
    container
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("container at depth {depth:?} has vacant index {index:?}"))
}

fn access_container_mut<'t>(
    value: &'t mut dyn Type,
    index: &str,
    depth: usize,
) -> anyhow::Result<&'t mut dyn Type> {
    let container = match value.type_mut() {
        TypeMut::Container(value) => value,
        _ => anyhow::bail!("expected container to index into at depth {depth:?}"),
    };

    let index = index.parse()?;
    container
        .get_mut(index)
        .ok_or_else(|| anyhow::anyhow!("container at depth {depth:?} has vacant index {index:?}"))
}

#[derive(Debug, PartialEq)]
enum Token<'p> {
    Dot,
    LBracket,
    RBracket,
    Ident(&'p str),
    End,
}

struct Lexer<'p> {
    path: &'p str,
    chars: Peekable<Chars<'p>>,
    pos: usize,
}

impl<'p> Lexer<'p> {
    pub fn new(path: &'p str) -> Self {
        Self {
            path,
            chars: path.chars().peekable(),
            pos: 0,
        }
    }

    pub fn lex_token(&mut self) -> Token<'p> {
        let start = self.pos;
        let mut pos = self.pos;

        // Opt out if we're already done with the path string.
        let next = self.chars.next();
        if next.is_none() {
            return Token::End;
        }

        pos += 1;
        // SAFETY: If `next` was `None`, we wouldn't be here.
        let result = match unsafe { next.unwrap_unchecked() } {
            '.' => Token::Dot,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            _ => {
                // We don't have much error handling to do, fortunately. So we
                // assume this is an ident, count chars until the next type of
                // token and delegate error handling to `Type` on access.
                while let Some(ch) = self.chars.peek() {
                    match ch {
                        '.' | '[' | ']' => break,
                        _ => {
                            self.chars.next();
                            pos += 1;
                        }
                    }
                }

                Token::Ident(&self.path[start..pos])
            }
        };

        self.pos = pos;
        result
    }
}

impl<'p> Iterator for Lexer<'p> {
    type Item = Token<'p>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lex_token() {
            Token::End => None,
            t => Some(t),
        }
    }
}
