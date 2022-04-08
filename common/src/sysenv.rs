pub mod serde_env {
    use std::env::VarError;
    use std::{io, ops};
    use std::io::Read;
    use serde::de::Error;

    pub fn from_reader<R: std::io::Read>(reader: R) -> serde_json::Result<String> {
        let ref mut buf = Default::default();
        let mut buf_reader = std::io::BufReader::new(reader);
        buf_reader.read_to_string(buf)
            .map_err(|err| serde_json::Error::custom("fail to read from reader"))
            .map(|_| replace_by_env(buf))
    }

    pub fn from_str(value: &str) -> String {
        replace_by_env(value)
    }

    fn replace_by_env(value: &str) -> String {
        let ref mut buf = value.to_string();
        let reg = regex::Regex::new("\\$\\{[^}]+\\}").unwrap();
        match reg.captures(value) {
            Some(captures) => captures
                .iter()
                .for_each(|matched| match matched {
                    Some(m) => match std::env::var(m
                        .as_str()[2..(m.end() - m.start() - 1)]
                        .to_string()) {
                        Ok(var) => buf.replace_range(m.range(), var.as_str()),
                        Err(_) => {}
                    },
                    _ => {}
                }),
            None => {}
        }

        buf.clone()
    }
}