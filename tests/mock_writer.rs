use std::{
    io,
    sync::{Arc, Mutex},
};
use tracing_subscriber::fmt::MakeWriter;

pub struct MockWriter {
    storage: Arc<Mutex<Vec<String>>>,
}

impl MockWriter {
    pub fn new(storage: Arc<Mutex<Vec<String>>>) -> Self {
        MockWriter { storage }
    }
}

impl<'a> io::Write for MockWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut storage_vec = self.storage.lock().unwrap();
        storage_vec.push(String::from_utf8_lossy(buf).to_string());
        Ok(storage_vec.last().unwrap().len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for MockWriter {
    type Writer = MockWriter;

    fn make_writer(&'a self) -> Self::Writer {
        MockWriter::new(self.storage.clone())
    }
}
