pub trait ResultExt<T, E> {
    fn or_panic(self, message: &str) -> T;
}

impl<T, E: std::fmt::Debug> ResultExt<T, E> for Result<T, E> {
    fn or_panic(self, message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(err) => panic!("{message}: {err:?}"),
        }
    }
}
