use crate::reader::Reader;


pub(crate) struct Question {
    pub(crate) name: String,
    pub(crate) qtype: u16,
    pub(crate) qclass: u16,
}

pub(crate) struct Query {
    pub(crate) questions: Vec<Question>,
}

impl Query {
    pub(crate) fn parse_query(buf: &[u8]) -> Option<Query> {
        let mut r = Reader::new(buf);
        let _id = r.u16()?;
        let flags = r.u16()?;
        if flags & 0x8000 != 0 {
            return None;
        }
        let qdcount = r.u16()? as usize;
        let _ancount = r.u16()?;
        let _nscount = r.u16()?;
        let _arcount = r.u16()?;

        let mut questions = Vec::with_capacity(qdcount);
        for _ in 0..qdcount {
            let name = r.name()?;
            let qtype = r.u16()?;
            let qclass = r.u16()?;
            questions.push(Question {
                name,
                qtype,
                qclass,
            });
        }
        Some(Query { questions })
    }
}