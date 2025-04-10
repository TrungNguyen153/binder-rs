#![allow(unsafe_op_in_unsafe_fn)]
#![feature(never_type)]

// https://www.synacktiv.com/en/publications/binder-transactions-in-the-bowels-of-the-linux-kernel.html
#[macro_use]
extern crate tracing;
mod macros;
mod binder;
pub mod error;
pub mod parcel;
pub mod parcelable;
#[cfg(feature = "binding-java")]
mod binding;
