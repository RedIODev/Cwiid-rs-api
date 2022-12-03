
pub trait Invert<T, E> {
    fn invert(self) -> Result<E, T>;
}

impl<T, E> Invert<T, E> for Result<T, E> {
    fn invert(self) -> Result<E, T> {
        match self {
            Ok(t) => Err(t),
            Err(e) => Ok(e)
        }
    }
}