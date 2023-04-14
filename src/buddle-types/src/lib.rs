//! Definitions of shared C++ types in Rust.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

pub mod raw {
    //! The raw generated types from codegen.
    #![allow(unused_imports)]

    include!(env!("BUDDLE_GENERATED_TYPES"));
}
