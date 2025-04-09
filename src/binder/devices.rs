/// Binder support devices
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum BinderDevice {
    /// /dev/binder
    #[default]
    Binder,
    /// /dev/hwbinder
    HwBinder,
    /// /dev/vndbinder
    VndBinder,
}

impl std::fmt::Display for BinderDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinderDevice::Binder => write!(f, "/dev/binder"),
            BinderDevice::HwBinder => write!(f, "/dev/hwbinder"),
            BinderDevice::VndBinder => write!(f, "/dev/vndbinder"),
        }
    }
}
