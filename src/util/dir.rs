use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

pub trait Labeled {
    const LABEL: &str;
}

pub trait Dir: From<PathBuf> + AsRef<Path> + Display {}

pub trait LabeledDir: Dir + Labeled {}
impl<D: Dir + Labeled> LabeledDir for D {}

#[macro_export]
macro_rules! dir {
    ($x:ident) => {
        #[derive(Debug, Display, Constructor, PartialEq, From, Eq, AsRef)]
        #[as_ref(Path)]
        #[display("{}", _0.display())]
        pub struct $x(PathBuf);
        impl $crate::util::dir::Dir for $x {}
    };
}

#[macro_export]
macro_rules! label {
    ($x:ident, $p:literal) => {
        impl $crate::util::dir::Labeled for $x {
            const LABEL: &str = $p;
        }
    };
}

#[macro_export]
macro_rules! labeled_dir {
    ($x:ident, $p:literal) => {
        dir!($x);
        label!($x, $p);
    };
}
