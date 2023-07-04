use std::sync::{Arc, Mutex};

pub type Parser<T, E> = Arc<Mutex<dyn OutputParser<Output = T, Error = E>>>;

pub trait OutputParser: Send {
    type Output;
    type Error;

    fn parse(&self, output: String) -> Result<Self::Output, Self::Error>;
}
