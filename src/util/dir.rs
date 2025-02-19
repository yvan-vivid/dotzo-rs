pub trait Labeled {
    const LABEL: &str;
}

#[macro_export]
macro_rules! dir {
    ($x:ident) => {
        #[derive(Debug, Constructor, PartialEq, From, Eq, AsRef)]
        #[as_ref(Path)]
        pub struct $x(PathBuf);
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
