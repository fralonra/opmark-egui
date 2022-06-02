use opmark::Parser as OpParser;

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

pub struct Parser {
    pub fullscreen: bool,
    pub parser: opmark::Parser,
    pub title: String,
}

impl Parser {
    pub fn new(s: String) -> Self {
        let mut fullscreen = false;
        let mut title = "OpMark Egui".to_owned();
        let mut origin_str = s.clone();

        if let Some(rest) = s.strip_prefix(&format!("---meta{}", LINE_ENDING)) {
            if let Some(meta_end) = rest.find(&format!("---{}", LINE_ENDING)) {
                let meta = rest[..meta_end].to_owned();
                for line in meta.split("\n") {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    match parts[0] {
                        "fullscreen" => {
                            if parts.len() > 1 && parts[1].trim() == "true" {
                                fullscreen = true;
                            }
                        }
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
            fullscreen,
            parser: OpParser::new(origin_str),
            title: title.to_owned(),
        }
    }
}
