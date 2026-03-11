
pub(crate) struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn u8(&mut self) -> Option<u8> {
        let b = self.buf.get(self.pos).copied()?;
        self.pos += 1;
        Some(b)
    }
    pub(crate) fn u16(&mut self) -> Option<u16> {
        let hi = self.u8()? as u16;
        let lo = self.u8()? as u16;
        Some((hi << 8) | lo)
    }
    pub(crate) fn name(&mut self) -> Option<String> {
        self.name_at(self.pos).map(|(s, end)| {
            self.pos = end;
            s
        })
    }

    fn name_at(&self, mut pos: usize) -> Option<(String, usize)> {
        let mut labels: Vec<String> = Vec::new();
        let mut jumped = false;
        let mut end_pos = pos;
        let mut safety = 0usize;

        loop {
            if safety > 128 {
                return None;
            }
            safety += 1;

            let len = *self.buf.get(pos)? as usize;
            if len == 0 {
                if !jumped {
                    end_pos = pos + 1;
                }
                break;
            } else if len & 0xC0 == 0xC0 {
                let lo = *self.buf.get(pos + 1)? as usize;
                let ptr = ((len & 0x3F) << 8) | lo;
                if !jumped {
                    end_pos = pos + 2;
                }
                jumped = true;
                pos = ptr;
            } else {
                pos += 1;
                let label = std::str::from_utf8(self.buf.get(pos..pos + len)?).ok()?;
                labels.push(label.to_owned());
                pos += len;
                if !jumped {
                    end_pos = pos;
                }
            }
        }
        Some((labels.join(".") + ".", end_pos))
    }
}