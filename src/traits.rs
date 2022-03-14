use std::io::Error;

pub trait WriteObject<W> {
    fn write_object(&mut self, write: W) -> Result<usize, Error>;
}

pub trait ReadObject<R> {
    fn read_object(&mut self) -> Option<R>;
}
