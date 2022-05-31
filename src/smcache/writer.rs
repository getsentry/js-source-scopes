impl Foo {
    /// Insert a string into this converter.
    ///
    /// If the string was already present, it is not added again. A newly added string
    /// is prefixed by its length as a `u32`. The returned `u32`
    /// is the offset into the `string_bytes` field where the string is saved.
    fn insert_string(
        string_bytes: &mut Vec<u8>,
        strings: &mut HashMap<String, u32>,
        s: &str,
    ) -> u32 {
        if s.is_empty() {
            return u32::MAX;
        }
        if let Some(&offset) = strings.get(s) {
            return offset;
        }
        let string_offset = string_bytes.len() as u32;
        let string_len = s.len() as u32;
        string_bytes.extend(string_len.to_ne_bytes());
        string_bytes.extend(s.bytes());
        // we should have written exactly `string_len + 4` bytes
        debug_assert_eq!(
            string_bytes.len(),
            string_offset as usize + string_len as usize + std::mem::size_of::<u32>(),
        );
        strings.insert(s.to_owned(), string_offset);
        string_offset
    }

    /// Serialize the converted data.
    ///
    /// This writes the SymCache binary format into the given [`Write`].
    pub fn serialize<W: Write>(mut self, writer: &mut W) -> std::io::Result<()> {}
}

struct WriteWrapper<W> {
    writer: W,
    position: usize,
}

impl<W: Write> WriteWrapper<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            position: 0,
        }
    }

    fn write<T>(&mut self, data: &[T]) -> std::io::Result<usize> {
        let pointer = data.as_ptr() as *const u8;
        let len = std::mem::size_of_val(data);
        // SAFETY: both pointer and len are derived directly from data/T and are valid.
        let buf = unsafe { std::slice::from_raw_parts(pointer, len) };
        self.writer.write_all(buf)?;
        self.position += len;
        Ok(len)
    }

    fn align(&mut self) -> std::io::Result<usize> {
        let buf = &[0u8; 7];
        let len = raw::align_to_eight(self.position);
        self.write(&buf[0..len])
    }
}
