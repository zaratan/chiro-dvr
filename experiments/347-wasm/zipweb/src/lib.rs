use std::io::{self, Write};

use wasm_bindgen::prelude::*;
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

struct Chunks {
    send: js_sys::Function,
}

impl Write for Chunks {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let array = js_sys::Uint8Array::from(bytes);
        self.send.call1(&JsValue::NULL, &array).map_err(|_| io::Error::other("send"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[wasm_bindgen]
pub struct Zipper(Option<ZipWriter<zip::write::StreamWriter<Chunks>>>);

#[wasm_bindgen]
impl Zipper {
    #[wasm_bindgen(constructor)]
    pub fn new(send: js_sys::Function) -> Self {
        Self(Some(ZipWriter::new_stream(Chunks { send })))
    }

    pub fn start(&mut self, name: &str, large: bool) -> Result<(), JsError> {
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .large_file(large);
        self.0.as_mut().ok_or_else(|| JsError::new("finished"))?.start_file(name, options)?;
        Ok(())
    }

    pub fn write(&mut self, bytes: &[u8]) -> Result<(), JsError> {
        self.0.as_mut().ok_or_else(|| JsError::new("finished"))?.write_all(bytes)?;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), JsError> {
        self.0.take().ok_or_else(|| JsError::new("finished"))?.finish()?;
        Ok(())
    }
}
