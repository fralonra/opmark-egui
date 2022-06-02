use opmark::Parser as OpParser;

pub struct Parser {
    pub parser: opmark::Parser,
    pub title: String,
}

impl Parser {
    pub fn new(s: String) -> Self {
        let mut title = "OpMark Egui".to_owned();
        let mut origin_str = s.clone();

        if let Some(rest) = s.strip_prefix("---meta\n") {
            if let Some(meta_end) = rest.find("---\n") {
                let meta = rest[..meta_end].to_owned();
                for line in meta.split("\n") {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    match parts[0] {
                        "title" => {
                            if parts.len() > 1 {
                                title = parts[1].trim().to_owned();
                            }
                        }
                        _ => {}
                    }
                }

                origin_str = rest[meta_end + 3..].to_owned();
            }
        }

        Self {
            parser: OpParser::new(origin_str),
            title: title.to_owned(),
        }
    }
}
