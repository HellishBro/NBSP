#[macro_export]
macro_rules! from_root {
    ($x:expr) => { concat!(env!("CARGO_MANIFEST_DIR"), "/", $x) };
}

#[macro_export]
macro_rules! asset {
    ($x:expr) => { from_root!(concat!("assets/", $x)) };
}