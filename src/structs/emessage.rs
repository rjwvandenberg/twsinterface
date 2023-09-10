
#[derive(Debug, Clone)]
pub struct EMessage {  
    buffer: String
}

impl EMessage {
    pub fn new() -> EMessage {
        EMessage{ buffer: String::with_capacity(4096) }
    }

    pub fn push(&mut self, text: impl AsRef<str>) {
        self.buffer.push_str(text.as_ref());
        self.buffer.push('\0');
    }

    pub fn consume(self) -> Vec<u8> {
        self.buffer.into_bytes()
    }
}

impl From<&[u8]> for EMessage {
    fn from(value: &[u8]) -> Self {
        EMessage { buffer: String::from_utf8(value.to_vec()).unwrap() }
    }
}