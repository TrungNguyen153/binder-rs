use crate::error::{BinderError, Result};

/// Interface stability promise
///
/// An interface can promise to be a stable vendor interface ([`Vintf`]), or
/// makes no stability guarantees ([`Local`]). [`Local`] is
/// currently the default stability.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stability {
    /// Default stability, visible to other modules in the same compilation
    /// context (e.g. modules on system.img)
    Local,
    Vendor,
    #[default]
    System,

    /// A Vendor Interface Object, which promises to be stable
    Vintf,
}

// Android 12 version uses "Category" as the stability format for passed on the wire lines,
// whereas other versions do not. Therefore, we can use the android_properties crate
// to determine the Android version and perform different handling accordingly.
// http://aospxref.com/android-11.0.0_r21/xref/frameworks/native/libs/binder/include/binder/Stability.h
// http://aospxref.com/android-12.0.0_r3/xref/frameworks/native/include/binder/Stability.h
// http://aospxref.com/android-13.0.0_r3/xref/frameworks/native/libs/binder/include/binder/Stability.h
// http://aospxref.com/android-14.0.0_r2/xref/frameworks/native/libs/binder/include/binder/Stability.h
impl From<Stability> for i32 {
    fn from(stability: Stability) -> i32 {
        use Stability::*;

        let stability = match stability {
            Local => 0,
            Vendor => 0b000011,
            System => 0b001100,
            Vintf => 0b111111,
        };

        #[cfg(target_os = "android")]
        if crate::get_android_version() == 12 {
            stability | 0x0c000000
        } else {
            stability
        }

        #[cfg(not(target_os = "android"))]
        stability
    }
}

impl TryFrom<i32> for Stability {
    type Error = BinderError;
    fn try_from(stability: i32) -> Result<Stability> {
        use Stability::*;
        match stability {
            stability if stability == Local.into() => Ok(Local),
            stability if stability == Vendor.into() => Ok(Vendor),
            stability if stability == System.into() => Ok(System),
            stability if stability == Vintf.into() => Ok(Vintf),
            _ => {
                error!("Stability value is invalid: {}", stability);
                Err(BinderError::BadValue)
            }
        }
    }
}
