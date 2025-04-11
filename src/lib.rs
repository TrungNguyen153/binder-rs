#![allow(unsafe_op_in_unsafe_fn)]
#![feature(never_type)]
#![feature(let_chains)]

// https://www.synacktiv.com/en/publications/binder-transactions-in-the-bowels-of-the-linux-kernel.html
#[macro_use]
extern crate tracing;
mod binder;
#[cfg(feature = "binding-java")]
mod binding;
pub mod error;
mod macros;
pub mod parcel;
pub mod service;
pub mod stability;

#[cfg(target_os = "android")]
pub fn get_android_version() -> u32 {
    // TODO
    15
}
