
pub(crate) struct Builder {
    buffer: Vec<u8>,
    ans_count: u16,
    add_count: u16,
    is_additional: bool,
}

impl Builder {
    pub(crate) fn response(id: u16) -> Self {
        let mut b = Self {
            buffer: Vec::with_capacity(512),
            ans_count: 0,
            add_count: 0,
            is_additional: false,
        };

        b.ext(id);
        b.ext(0x8400);
        b.ext(0);
        b.ext(0);
        b.ext(0);
        b.ext(0);

        b
    }

    fn ext(&mut self, b: u16) {
        self.buffer.extend_from_slice(&b.to_be_bytes());
    }
    fn name(&mut self, name: &str) {
        for label in name.trim_end_matches('.').split('.') {
            let bytes = label.as_bytes();
            self.buffer.push(bytes.len() as u8);
            self.buffer.extend_from_slice(bytes);
        }
        self.buffer.push(0);
    }
    pub(crate) fn additional(&mut self) {
        self.is_additional = true;
    }

    fn write_header(&mut self, name: &str, rtype: u16, class: u16, ttl: u32, rdlen: u16) {
        if self.is_additional {
            self.add_count += 1;
        } else {
            self.ans_count += 1;
        }

        self.name(name);
        self.ext(rtype);
        self.ext(class);
        self.buffer.extend_from_slice(&ttl.to_be_bytes());
        self.ext(rdlen);
    }

    fn with_name_data(&mut self, owner: &str, target: &str, rtype: u16, class: u16, ttl: u32) {
        let mut b = Vec::new();

        for label in target.trim_end_matches('.').split('.') {
            b.push(label.len() as u8);
            b.extend_from_slice(label.as_bytes());
        }
        b.push(0);

        self.write_header(owner, rtype, class, ttl, b.len() as u16);
        self.buffer.extend_from_slice(&b);
    }

    pub(crate) fn ptr(&mut self, sname: &str, stype: &str, ttl: u32) {
        self.with_name_data(sname, stype, 12, 0x8001, ttl);
    }

    pub(crate) fn srv(&mut self, sname: &str, target: &str, priority: u16, weight: u16, port: u16, ttl: u32) {
        let mut b = Vec::new();

        for label in target.trim_end_matches('.').split('.') {
            b.push(label.len() as u8);
            b.extend_from_slice(label.as_bytes());
        }

        b.push(0);

        let rdlen = b.len() + 6;

        self.write_header(sname, 33, 0x8001, ttl, rdlen as u16);

        self.ext(priority);
        self.ext(weight);
        self.ext(port);

        self.buffer.extend_from_slice(&b);
    }

    pub(crate) fn txt(&mut self, sname: &str, txt: &[(&str, &str)], ttl: u32) {
        let mut data = Vec::new();

        if txt.is_empty() {
            data.push(0u8);
        } else {
            for (key, value) in txt {
                let s = format!("{}={}", key, value);
                let sb = s.as_bytes();
                data.push(sb.len() as u8);
                data.extend_from_slice(sb);
            }
        }

        self.write_header(sname, 16, 0x8001, ttl, data.len() as u16);
        self.buffer.extend_from_slice(&data);
    }

    pub(crate) fn a(&mut self, hostname: &str, ip: std::net::Ipv4Addr, ttl: u32) {
        self.write_header(hostname, 1, 0x8001, ttl, 4);
        self.buffer.extend_from_slice(&ip.octets());
    }

    pub(crate) fn build(mut self) -> Vec<u8> {
        self.buffer[6] = (self.ans_count >> 8) as u8;
        self.buffer[7] = (self.ans_count & 0xFF) as u8;

        self.buffer[10] = (self.add_count >> 8) as u8;
        self.buffer[11] = (self.add_count & 0xFF) as u8;
        self.buffer
    }
}