mod fs;
mod templates;

use fs::{mkdir, mkfile, mvfile, rm};
use opmark::{
    mark::{
        AlignHorizontal, Heading, IndentLevel, Listing, Mark, SeparatorDir, StyleImage, StyleText,
    },
    Parser,
};
use std::{
    env::{current_dir, set_current_dir},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use templates::{
    CARGO_TOML_CONTENT, SRC_APP_1_CONTENT, SRC_APP_2_CONTENT, SRC_APP_3_CONTENT,
    SRC_IMPORT_CONTENT, SRC_MAIN_CONTENT,
};

fn code_block(code_str: &String, _language: &Option<String>, code: &mut String) -> bool {
    code.push_str("let where_to_put_background = ui.painter().add(egui::Shape::Noop);");
    code.push_str(&format!(
        "let mut rect = ui.monospace(\"{}\").rect;",
        code_str
    ));
    code.push_str("rect = rect.expand(NORMAL_SPACING_X);");
    code.push_str("rect.max.x = ui.max_rect().max.x;");
    code.push_str("let code_bg_color = ui.visuals().code_bg_color;");
    code.push_str("ui.painter().set(");
    code.push_str("where_to_put_background,");
    code.push_str("egui::Shape::rect_filled(rect, 2.0, code_bg_color),");
    code.push_str(");");
    true
}

fn image(key: &String, title: &String, style: &StyleImage, code: &mut String) -> bool {
    code.push_str(&format!(
        "let texture = self.textures.get(&\"{}\".to_string()).unwrap();",
        key
    ));

    match style.width {
        Some(width) => {
            code.push_str(&format!("let width = {}.0;", width));
            match style.height {
                Some(height) => {
                    code.push_str(&format!("let height = {}.0;", height));
                }
                None => {
                    code.push_str("let height = width / texture.size.x * texture.size.y;");
                }
            };
        }
        None => code.push_str("let width = texture.size.x;"),
    };
    match style.align_h {
        AlignHorizontal::Auto | AlignHorizontal::Left => {
            code.push_str("let layout = egui::Layout::left_to_right();");
        }
        AlignHorizontal::Right => {
            code.push_str("let layout = egui::Layout::right_to_left();");
        }
        AlignHorizontal::Center => {
            code.push_str(
                "let layout = egui::Layout::centered_and_justified(egui::Direction::LeftToRight);",
            );
        }
    };

    code.push_str("ui.with_layout(layout, |ui| {");
    code.push_str(&format!(
        "ui.image(texture.id, egui::Vec2::new(width, height)).on_hover_text(\"{}\");",
        title
    ));
    code.push_str("});");
    true
}

fn indent(indent_level: &IndentLevel, code: &mut String) {
    let indent = indent_level.to_int() as f32;
    code.push_str(&format!("indent({}.0, ui);", indent.to_string()));
}

fn label(text: &str, style: &StyleText, code_str: &mut String) -> bool {
    let StyleText {
        bold,
        code,
        heading,
        hyperlink,
        italics,
        small,
        strikethrough,
        underline,
        ..
    } = &*style;

    let mut line_break = false;
    if hyperlink.is_empty() {
        code_str.push_str(&format!(
            "let rich_text = egui::RichText::new(\"{}\");",
            text
        ));
        if *bold {
            code_str.push_str("let rich_text = rich_text.strong();");
        }
        if *code {
            code_str.push_str("let rich_text = rich_text.code();");
        }
        if *italics {
            code_str.push_str("let rich_text = rich_text.italics();");
        }
        if *small {
            code_str.push_str("let rich_text = rich_text.small();");
        }
        if *strikethrough {
            code_str.push_str("let rich_text = rich_text.strikethrough();");
        }
        if *underline {
            code_str.push_str("let rich_text = rich_text.underline();");
        }
        match heading {
            Heading::H1 => code_str.push_str("let rich_text = rich_text.heading().strong();"),
            Heading::H2 => code_str.push_str("let rich_text = rich_text.heading();"),
            _ => {}
        };
        match heading {
            Heading::None => {}
            _ => line_break = true,
        }
        code_str.push_str("ui.label(rich_text);");
    } else {
        code_str.push_str(&format!(
            "ui.hyperlink_to(\"{}\", \"{}\");",
            text, hyperlink
        ));
    }
    line_break
}

fn line_break(code: &mut String) {
    code.push_str("outer_ui.end_row();");
}

fn new_line(code: &mut String) -> bool {
    code.push_str(
        "ui.allocate_exact_size(egui::vec2(0.0, NEW_LINE_HEIGHT), egui::Sense::hover());",
    );
    true
}

fn ordered_listing(
    n: u8,
    text: &str,
    style: &StyleText,
    indent_level: &IndentLevel,
    code: &mut String,
) -> bool {
    indent(indent_level, code);
    code.push_str("ui.spacing_mut().item_spacing.x = BIG_SPACING_X;");
    label(&format!("{}.", n), style, code);
    code.push_str("ui.spacing_mut().item_spacing.x = NORMAL_SPACING_X;");
    label(&text, style, code);
    true
}

fn quote(text: &str, style: &StyleText, code: &mut String) -> bool {
    code.push_str("ui.spacing_mut().item_spacing.x = BIG_SPACING_X;");
    code.push_str("let (rect, _) = ui.allocate_exact_size(");
    code.push_str("egui::vec2(INDENT_GRID_SIZE, INDENT_GRID_SIZE),");
    code.push_str("egui::Sense::hover(),");
    code.push_str(");");
    code.push_str("let rect = rect.expand2(ui.style().spacing.item_spacing * 0.5);");
    code.push_str("ui.painter().line_segment(");
    code.push_str("[rect.center_top(), rect.center_bottom()],");
    code.push_str("(1.0, ui.visuals().weak_text_color()),");
    code.push_str(");");
    code.push_str("ui.spacing_mut().item_spacing.x = NORMAL_SPACING_X;");
    label(&text, style, code);
    true
}

fn replace_str(src: &String, from: &str, to: &str) -> String {
    src.replace(&format!("[[[{}]]]", from), to)
}

fn seperator(dir: &SeparatorDir, code: &mut String) -> bool {
    match dir {
        SeparatorDir::Horizontal => {
            code.push_str("ui.add(egui::Separator::default().horizontal());");
        }
        SeparatorDir::Vertical => {
            code.push_str("ui.separator();");
        }
    };
    true
}

fn transition(transition_border: usize, code: &mut String) {
    if transition_border > 0 {
        code.push_str(&format!("if self.scene_idx < {} {{", transition_border));
        code.push_str("ui.set_visible(false);");
        code.push_str("}");
    }
}

fn unordered_listing(
    text: &str,
    style: &StyleText,
    indent_level: &IndentLevel,
    code: &mut String,
) -> bool {
    let indent = indent_level.to_int() as f32;
    code.push_str("ui.spacing_mut().item_spacing.x = BIG_SPACING_X;");
    code.push_str(&format!("let rect = indent({}.0, ui);", indent));
    code.push_str("let mark_center = egui::Pos2::new(rect.right(), rect.center().y);");
    code.push_str("let mark_size = 4.0;");
    code.push_str("let color = ui.visuals().strong_text_color();");
    match indent_level {
        IndentLevel::None => {
            code.push_str("ui.painter().circle_filled(mark_center, mark_size / 2.0, color);");
        }
        IndentLevel::I1 => {
            code.push_str("ui.painter().circle_stroke(mark_center, mark_size / 2.0, egui::Stroke::new(0.5, color));");
        }
        IndentLevel::I2 => {
            code.push_str(
                "ui.painter().rect_filled(egui::Rect::from_center_size(mark_center, egui::vec2(mark_size, mark_size)), 0.0, color);",
            );
        }
        _ => {
            code.push_str(
                "ui.painter().rect_stroke(egui::Rect::from_center_size(mark_center, egui::vec2(mark_size, mark_size)), 0.0, egui::Stroke::new(0.5, color));",
            );
        }
    }
    code.push_str("ui.spacing_mut().item_spacing.x = NORMAL_SPACING_X;");
    label(&text, style, code);
    true
}

pub struct Builder {
    pages: Vec<(Mark, usize, usize)>,
}

impl Builder {
    pub fn new(iter: Parser) -> Self {
        Builder {
            pages: Parser::into_pages(iter),
        }
    }

    pub fn build(&self, path: &Path, output: &Path) {
        use std::io::{self, Write};

        let root = current_dir().expect(&format!("[ERROR] failed to get current directory"));
        let root = root.as_path();
        set_current_dir(path).expect(&format!(
            "[ERROR] failed to set current directory to `{}`",
            path.display()
        ));

        let o = Command::new("cargo")
            .args(["build", "--release"])
            .output()
            .expect(&format!(
                "[ERROR] failed to execute process `cargo run --release` at {}",
                path.display()
            ));
        io::stdout().write_all(&o.stdout).unwrap();
        io::stderr().write_all(&o.stderr).unwrap();

        let build_release = PathBuf::from(path)
            .join("target")
            .join("release")
            .join("opmark");

        set_current_dir(root).expect(&format!(
            "[ERROR] failed to set current directory to `{}`",
            root.display()
        ));

        mvfile(build_release.as_path(), output);
        rm(path);
    }

    pub fn code(&self) -> PathBuf {
        let src = self.generate_code();

        let now = SystemTime::now();
        let root = format!(
            ".opmark-out-{}",
            now.duration_since(UNIX_EPOCH).unwrap().as_nanos()
        );
        let root_path = Path::new(&root);

        mkdir(&root_path);
        mkfile(&root_path.join("Cargo.toml"), CARGO_TOML_CONTENT.as_bytes());

        let src_path = root_path.join("src");
        mkdir(&src_path);
        mkfile(&src_path.join("main.rs"), &src.as_bytes());

        PathBuf::from(root_path)
    }

    fn generate_code(&self) -> String {
        let mut code = String::new();
        code.push_str(SRC_IMPORT_CONTENT);
        code.push_str(&replace_str(
            &SRC_APP_1_CONTENT.to_owned(),
            "name",
            "\"opmark\"",
        ));
        code.push_str(&self.generate_code_setup());
        code.push_str(&SRC_APP_2_CONTENT.to_owned());
        code.push_str(&self.generate_code_update());
        code.push_str(SRC_MAIN_CONTENT);
        code
    }

    fn generate_code_setup(&self) -> String {
        let mut code = String::new();
        let mut resources = vec![];
        for page in &self.pages {
            if let (Mark::Page(transitions), ..) = page {
                for transition in transitions {
                    if let Mark::Transition(.., marks) = transition {
                        for mark in marks {
                            if let Mark::Image(key, ..) = mark {
                                if resources.contains(key) {
                                    continue;
                                }
                                let key_path = PathBuf::from("..").join("..").join(key);
                                code.push_str(&format!(
                                    "let image_data = include_bytes!(\"{}\");",
                                    key_path.as_path().display()
                                ));
                                code.push_str("let texture = load_texture(image_data, frame).expect(\"Failed to load texture\");");
                                code.push_str(&format!(
                                    "self.textures.insert(\"{}\".to_string(), texture);",
                                    key
                                ));
                                resources.push(key.to_string());
                            }
                        }
                    }
                }
            }
        }
        code
    }

    fn generate_code_update(&self) -> String {
        let mut code = String::new();
        let mut max_scene_idx = 0;
        for page in &self.pages {
            if let (Mark::Page(transitions), max_transition_idx, _) = page {
                let current_page_idx = max_scene_idx;
                let next_page_idx = current_page_idx + max_transition_idx + 1;
                max_scene_idx += max_transition_idx + 1;
                code.push_str(&format!(
                    "if self.scene_idx >= {} && self.scene_idx < {} {{",
                    current_page_idx, next_page_idx
                ));

                for t in transitions {
                    if let Mark::Transition(order, marks) = t {
                        for mark in marks {
                            code.push_str("outer_ui.horizontal(|ui| {");
                            let transition_border = current_page_idx + order;
                            transition(transition_border, &mut code);

                            let need_line_break = match mark {
                                Mark::CodeBlock(c, language) => code_block(c, language, &mut code),
                                Mark::Image(key, title, style) => {
                                    image(key, title, style, &mut code)
                                }
                                Mark::NewLine => new_line(&mut code),
                                Mark::Separator(dir) => seperator(dir, &mut code),
                                Mark::Text(text, style) => match &style.listing {
                                    Listing::Ordered(n, indent) => {
                                        ordered_listing(*n, &text, style, indent, &mut code)
                                    }
                                    Listing::Unordered(indent) => {
                                        unordered_listing(&text, style, indent, &mut code)
                                    }
                                    _ => {
                                        if !style.quote {
                                            label(&text, style, &mut code)
                                        } else {
                                            quote(&text, style, &mut code)
                                        }
                                    }
                                },
                                _ => false,
                            };

                            code.push_str("});");
                            if need_line_break {
                                line_break(&mut code);
                            }
                        }
                    }
                }

                code.push_str("}");
            }
        }
        code.push_str(&replace_str(
            &SRC_APP_3_CONTENT.to_owned(),
            "max_scene_idx",
            &max_scene_idx.to_string(),
        ));
        code
    }
}
