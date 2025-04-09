#![allow(unsafe_op_in_unsafe_fn)]

// https://www.synacktiv.com/en/publications/binder-transactions-in-the-bowels-of-the-linux-kernel.html
#[macro_use]
extern crate tracing;
mod binder;
pub mod error;
pub mod parcel;
pub mod parcelable;
