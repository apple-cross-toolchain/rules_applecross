//! Metal shaders without a Mac: MSL compiled to AIR with LLVM 15 and our own
//! stdlib, packaged as a metallib. Learned black-box from Xcode's output for our
//! own probe shaders (README.md, FORMAT.md).

pub mod air;
pub mod ast;
pub mod bitcode;
pub mod container;
pub mod descriptors;
pub mod driver;
pub mod entry;
pub mod intrinsics;
pub mod ir;
pub mod library;
pub mod rewrite;
pub mod target;
pub mod text;
pub mod types;
