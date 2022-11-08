//! Recursive access to nested values in reflected types.

use std::{iter::Peekable, num::ParseIntError, str::Chars};

use thiserror::Error;

use crate::{Type, TypeMut, TypeRef};

/// Errors that may occur during path resolution via [`PathAccess`].
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum PathError<'p> {
    /// Occurs when resolution of a class property
    /// identifier at a specific depth failed.
    #[error("Expected structure at depth {depth:?}")]
    ExpectedStructure { depth: usize },
    /// Occurs when resolution of a container index at
    /// a specific depth failed.
    #[error("Expected container to index into at depth {depth:?}")]
    ExpectedContainer { depth: usize },
    /// A class does not have a property that matches the
    /// name of the identifier.
    #[error("Value at depth {depth:?} does not have a field named {ident}")]
    InvalidIdent { ident: &'p str, depth: usize },
    /// A container stores no element at the supplied index.
    #[error("Container at depth {depth:?} has vacant index {index:?}")]
    InvalidContainerIndex { index: usize, depth: usize },
    /// A generic lexer error for the supplied path string.
    #[error("Path lexer failed at position {position:?}: {error}")]
    LexerFailed {
        error: &'static str,
        position: usize,
    },
    /// Identifier that was assumed to be a container index
    /// is not a number.
    #[error("Failed to parse container index as usize: {0}")]
    InvalidIndex(#[from] ParseIntError),
    /// A reflected value was attempted to be downcast into
    /// an incorrect concrete type.
    #[error("Cannot downcast path element into incorrect type")]
    InvalidDowncast,
}

/// Dynamic access into nested data structures by paths.
///
/// Paths are strings in the form of `"container[1].melement"`
/// to access the `my_element` property of the object stored
/// at index `1` of container `my_container`.
///
/// In that spirit, path strings try to approximate actual
/// Rust syntax to resolve a value as closely as possible.
pub trait PathAccess: Type {
    /// Attempts to resolve a given path into this value
    /// and returns an immutable reference on success.
    fn path<'p>(&self, path: &'p str) -> Result<&dyn Type, PathError<'p>>;

    /// Attempts to resolve a given path into this value
    /// and returns a mutable reference on success.
    fn path_mut<'p>(&mut self, path: &'p str) -> Result<&mut dyn Type, PathError<'p>>;

    /// Resolves a path with [`PathAccess::path`] and attempts
    /// to downcast into a concrete type.
    fn path_as<'p, T: Type>(&self, path: &'p str) -> Result<&T, PathError<'p>> {
        self.path(path)
            .and_then(|p| p.downcast_ref().ok_or(PathError::InvalidDowncast))
    }

    /// Resolves a path with [`PathAccess::path_mut`] and
    /// attempts to downcast into a concrete type.
    fn path_as_mut<'p, T: Type>(&mut self, path: &'p str) -> Result<&mut T, PathError<'p>> {
        self.path_mut(path)
            .and_then(|p| p.downcast_mut().ok_or(PathError::InvalidDowncast))
    }
}

impl PathAccess for dyn Type {
    fn path<'p>(&self, path: &'p str) -> Result<&dyn Type, PathError<'p>> {
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
                        return Err(PathError::LexerFailed {
                            error: "'.' must be followed by a struct field ident",
                            position: lexer.pos,
                        });
                    }
                }

                Token::LBracket => {
                    if let Some(Token::Ident(ident)) = lexer.next() {
                        current = access_container(current, ident, depth)?;
                    } else {
                        return Err(PathError::LexerFailed {
                            error: "'[' must be followed by an identifier",
                            position: lexer.pos,
                        });
                    }

                    if lexer.next() != Some(Token::RBracket) {
                        return Err(PathError::LexerFailed {
                            error: "expected token ']'",
                            position: lexer.pos,
                        });
                    }

                    depth += 1;
                }

                Token::RBracket => {
                    return Err(PathError::LexerFailed {
                        error: "unexpected token ']' encountered",
                        position: lexer.pos,
                    })
                }

                Token::Ident(ident) => {
                    if start {
                        current = access_field(current, ident, depth)?;
                        depth += 1;
                    } else {
                        return Err(PathError::LexerFailed {
                            error: "expected '.' or '[' before ident",
                            position: lexer.pos,
                        });
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

    fn path_mut<'p>(&mut self, path: &'p str) -> Result<&mut dyn Type, PathError<'p>> {
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
                        return Err(PathError::LexerFailed {
                            error: "'.' must be followed by a struct field ident",
                            position: lexer.pos,
                        });
                    }
                }

                Token::LBracket => {
                    if let Some(Token::Ident(ident)) = lexer.next() {
                        current = access_container_mut(current, ident, depth)?;
                    } else {
                        return Err(PathError::LexerFailed {
                            error: "'[' must be followed by an identifier",
                            position: lexer.pos,
                        });
                    }

                    if lexer.next() != Some(Token::RBracket) {
                        return Err(PathError::LexerFailed {
                            error: "expected token ']'",
                            position: lexer.pos,
                        });
                    }

                    depth += 1;
                }

                Token::RBracket => {
                    return Err(PathError::LexerFailed {
                        error: "unexpected token ']' encountered",
                        position: lexer.pos,
                    })
                }

                Token::Ident(ident) => {
                    if start {
                        current = access_field_mut(current, ident, depth)?;
                        depth += 1;
                    } else {
                        return Err(PathError::LexerFailed {
                            error: "expected '.' or '[' before ident",
                            position: lexer.pos,
                        });
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
    fn path<'p>(&self, path: &'p str) -> Result<&dyn Type, PathError<'p>> {
        <dyn Type>::path(self, path)
    }

    fn path_mut<'p>(&mut self, path: &'p str) -> Result<&mut dyn Type, PathError<'p>> {
        <dyn Type>::path_mut(self, path)
    }
}

fn access_field<'p, 't>(
    value: &'t dyn Type,
    ident: &'p str,
    depth: usize,
) -> Result<&'t dyn Type, PathError<'p>> {
    let cls = match value.type_ref() {
        TypeRef::Class(value) => value,
        _ => return Err(PathError::ExpectedStructure { depth }),
    };

    let list = cls.property_list();
    list.property(ident)
        .and_then(|view| cls.property(view))
        .ok_or(PathError::InvalidIdent { ident, depth })
}

fn access_field_mut<'p, 't>(
    value: &'t mut dyn Type,
    ident: &'p str,
    depth: usize,
) -> Result<&'t mut dyn Type, PathError<'p>> {
    let cls = match value.type_mut() {
        TypeMut::Class(value) => value,
        _ => return Err(PathError::ExpectedStructure { depth }),
    };

    let list = cls.property_list();
    list.property(ident)
        .and_then(|view| cls.property_mut(view))
        .ok_or(PathError::InvalidIdent { ident, depth })
}

fn access_container<'p, 't>(
    value: &'t dyn Type,
    index: &'p str,
    depth: usize,
) -> Result<&'t dyn Type, PathError<'p>> {
    let container = match value.type_ref() {
        TypeRef::Container(value) => value,
        _ => return Err(PathError::ExpectedContainer { depth }),
    };

    let index = index.parse()?;
    container
        .get(index)
        .ok_or(PathError::InvalidContainerIndex { index, depth })
}

fn access_container_mut<'p, 't>(
    value: &'t mut dyn Type,
    index: &'p str,
    depth: usize,
) -> Result<&'t mut dyn Type, PathError<'p>> {
    let container = match value.type_mut() {
        TypeMut::Container(value) => value,
        _ => return Err(PathError::ExpectedContainer { depth }),
    };

    let index = index.parse()?;
    container
        .get_mut(index)
        .ok_or(PathError::InvalidContainerIndex { index, depth })
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
