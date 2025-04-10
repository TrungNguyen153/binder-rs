
#[macro_export]
macro_rules! pack_chars {
    ($c1:expr, $c2:expr, $c3:expr, $c4:expr) => {
        ((($c1 as u32) << 24) | (($c2 as u32) << 16) | (($c3 as u32) << 8) | ($c4 as u32))
    };
}

#[macro_export]
macro_rules! _iow {
    ($c1:expr, $c2:expr, $c3:expr) => {
        ((0x40 << 24) | (($c3 as u32) << 16) | (($c1 as u32) << 8) | ($c2 as u32))
    };
}

#[macro_export]
macro_rules! _ior {
    ($c1:expr, $c2:expr, $c3:expr) => {
        ((0x80 << 24) | (($c3 as u32) << 16) | (($c1 as u32) << 8) | ($c2 as u32))
    };
}

#[macro_export]
macro_rules! _io {
    ($c1:expr, $c2:expr) => {
        ((($c1 as u32) << 8) | ($c2 as u32))
    };
}
