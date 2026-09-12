const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

#[inline]
fn rotr(value: u32, bits: u32) -> u32 {
    value.rotate_right(bits)
}

pub fn sha256_hex(input: &[u8]) -> String {
    let mut data = input.to_vec();
    let bit_len = (data.len() as u64) * 8;
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());

    let mut hash = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in data.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, word) in words[..16].iter_mut().enumerate() {
            let start = index * 4;
            *word = u32::from_be_bytes(chunk[start..start + 4].try_into().unwrap());
        }
        for index in 16..64 {
            let s0 =
                rotr(words[index - 15], 7) ^ rotr(words[index - 15], 18) ^ (words[index - 15] >> 3);
            let s1 =
                rotr(words[index - 2], 17) ^ rotr(words[index - 2], 19) ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (
            hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
        );
        for index in 0..64 {
            let s1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let s0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(majority);
            (h, g, f, e, d, c, b, a) = (
                g,
                f,
                e,
                d.wrapping_add(temp1),
                c,
                b,
                a,
                temp1.wrapping_add(temp2),
            );
        }
        hash[0] = hash[0].wrapping_add(a);
        hash[1] = hash[1].wrapping_add(b);
        hash[2] = hash[2].wrapping_add(c);
        hash[3] = hash[3].wrapping_add(d);
        hash[4] = hash[4].wrapping_add(e);
        hash[5] = hash[5].wrapping_add(f);
        hash[6] = hash[6].wrapping_add(g);
        hash[7] = hash[7].wrapping_add(h);
    }
    let mut output = String::with_capacity(64);
    for word in hash {
        output.push_str(&format!("{word:08x}"));
    }
    output
}

#[derive(Debug)]
enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(String),
    Bool(bool),
    Null,
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> JsonParser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }
    fn skip_ws(&mut self) {
        while self
            .bytes
            .get(self.index)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            self.index += 1;
        }
    }
    fn value(&mut self) -> Result<JsonValue, ()> {
        self.skip_ws();
        match self.bytes.get(self.index).copied() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(JsonValue::String(self.string()?)),
            Some(b't') if self.take(b"true") => Ok(JsonValue::Bool(true)),
            Some(b'f') if self.take(b"false") => Ok(JsonValue::Bool(false)),
            Some(b'n') if self.take(b"null") => Ok(JsonValue::Null),
            Some(byte) if byte == b'-' || byte.is_ascii_digit() => self.number(),
            _ => Err(()),
        }
    }
    fn take(&mut self, token: &[u8]) -> bool {
        if self.bytes.get(self.index..self.index + token.len()) == Some(token) {
            self.index += token.len();
            true
        } else {
            false
        }
    }
    fn string(&mut self) -> Result<String, ()> {
        if self.bytes.get(self.index) != Some(&b'"') {
            return Err(());
        }
        self.index += 1;
        let mut output = String::new();
        while let Some(byte) = self.bytes.get(self.index).copied() {
            self.index += 1;
            match byte {
                b'"' => return Ok(output),
                b'\\' => {
                    let escaped = self.bytes.get(self.index).copied().ok_or(())?;
                    self.index += 1;
                    match escaped {
                        b'"' => output.push('"'),
                        b'\\' => output.push('\\'),
                        b'/' => output.push('/'),
                        b'b' => output.push('\u{0008}'),
                        b'f' => output.push('\u{000c}'),
                        b'n' => output.push('\n'),
                        b'r' => output.push('\r'),
                        b't' => output.push('\t'),
                        b'u' => {
                            let hex = std::str::from_utf8(
                                self.bytes.get(self.index..self.index + 4).ok_or(())?,
                            )
                            .map_err(|_| ())?;
                            self.index += 4;
                            output.push(
                                char::from_u32(u32::from_str_radix(hex, 16).map_err(|_| ())?)
                                    .ok_or(())?,
                            );
                        }
                        _ => return Err(()),
                    }
                }
                byte if byte >= 0x20 => output.push(byte as char),
                _ => return Err(()),
            }
        }
        Err(())
    }
    fn object(&mut self) -> Result<JsonValue, ()> {
        self.index += 1;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.bytes.get(self.index) == Some(&b'}') {
            self.index += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            self.skip_ws();
            if self.bytes.get(self.index) != Some(&b':') {
                return Err(());
            }
            self.index += 1;
            let value = self.value()?;
            entries.push((key, value));
            self.skip_ws();
            match self.bytes.get(self.index) {
                Some(b',') => self.index += 1,
                Some(b'}') => {
                    self.index += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => return Err(()),
            }
        }
    }
    fn array(&mut self) -> Result<JsonValue, ()> {
        self.index += 1;
        let mut values = Vec::new();
        self.skip_ws();
        if self.bytes.get(self.index) == Some(&b']') {
            self.index += 1;
            return Ok(JsonValue::Array(values));
        }
        loop {
            values.push(self.value()?);
            self.skip_ws();
            match self.bytes.get(self.index) {
                Some(b',') => self.index += 1,
                Some(b']') => {
                    self.index += 1;
                    return Ok(JsonValue::Array(values));
                }
                _ => return Err(()),
            }
        }
    }
    fn number(&mut self) -> Result<JsonValue, ()> {
        let start = self.index;
        while self
            .bytes
            .get(self.index)
            .is_some_and(|byte| byte.is_ascii_digit() || b"+-.eE".contains(byte))
        {
            self.index += 1;
        }
        Ok(JsonValue::Number(
            String::from_utf8(self.bytes[start..self.index].to_vec()).map_err(|_| ())?,
        ))
    }
}

fn emit_json(value: &JsonValue, output: &mut String) {
    match value {
        JsonValue::Object(entries) => {
            let mut sorted = entries.iter().collect::<Vec<_>>();
            sorted.sort_by(|left, right| left.0.cmp(&right.0));
            output.push('{');
            for (index, (key, value)) in sorted.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                emit_json(&JsonValue::String((*key).clone()), output);
                output.push(':');
                emit_json(value, output);
            }
            output.push('}');
        }
        JsonValue::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                emit_json(value, output);
            }
            output.push(']');
        }
        JsonValue::String(value) => {
            output.push('"');
            for byte in value.bytes() {
                match byte {
                    b'"' => output.push_str("\\\""),
                    b'\\' => output.push_str("\\\\"),
                    0x08 => output.push_str("\\b"),
                    0x0c => output.push_str("\\f"),
                    b'\n' => output.push_str("\\n"),
                    b'\r' => output.push_str("\\r"),
                    b'\t' => output.push_str("\\t"),
                    byte if byte < 0x20 => output.push_str(&format!("\\u{byte:04x}")),
                    byte => output.push(byte as char),
                }
            }
            output.push('"');
        }
        JsonValue::Number(value) => output.push_str(value),
        JsonValue::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        JsonValue::Null => output.push_str("null"),
    }
}

pub fn canonical_json_bytes(input: &[u8]) -> Result<Vec<u8>, ()> {
    let mut parser = JsonParser::new(input);
    let value = parser.value()?;
    parser.skip_ws();
    if parser.index != input.len() {
        return Err(());
    }
    let mut output = String::new();
    emit_json(&value, &mut output);
    output.push('\n');
    Ok(output.into_bytes())
}
