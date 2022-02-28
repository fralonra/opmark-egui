mod texture;

use eframe::{egui, epi};
use opmark::{
    mark::{
        AlignHorizontal, Heading, IndentLevel, Listing, Mark, SeparatorDir, StyleImage, StyleText,
    },
    Parser,
};
use std::collections::HashMap;
use std::path::Path;
use texture::{load_image, Texture};

const NORMAL_SPACING_X: f32 = 2.0;
const NORMAL_SPACING_Y: f32 = 2.0;
const BIG_SPACING_X: f32 = 8.0;
const INDENT_GRID_SIZE: f32 = 16.0;
const NEW_LINE_HEIGHT: f32 = 16.0;

pub struct App<'a> {
    current_page_idx: usize,
    pages: Vec<(Mark, usize, usize)>, // Vec<(marks, max_transition_idx, transition_idx)>
    title: &'a str,
    textures: HashMap<String, Texture>,
}

fn code_block(code: &String, _language: &Option<String>, ui: &mut egui::Ui) -> bool {
    let where_to_put_background = ui.painter().add(egui::Shape::Noop);
    let mut rect = ui.monospace(code).rect;
    rect = rect.expand(NORMAL_SPACING_X);
    rect.max.x = ui.max_rect().max.x;
    let code_bg_color = ui.visuals().code_bg_color;
    ui.painter().set(
        where_to_put_background,
        egui::Shape::rect_filled(rect, 2.0, code_bg_color),
    );
    true
}

fn image(title: &String, texture: &Texture, style: &StyleImage, ui: &mut egui::Ui) -> bool {
    let mut height = texture.size.y;
    let width = match style.width {
        Some(width) => {
            height = match style.height {
                Some(height) => height,
                None => width / texture.size.x * texture.size.y,
            };
            width
        }
        None => texture.size.x,
    };
    let layout = match style.align_h {
        AlignHorizontal::Auto | AlignHorizontal::Left => egui::Layout::left_to_right(),
        AlignHorizontal::Right => egui::Layout::right_to_left(),
        AlignHorizontal::Center => {
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
        }
    };
    ui.with_layout(layout, |ui| {
        ui.image(texture.id, egui::Vec2::new(width, height))
            .on_hover_text(title);
    });
    true
}

fn indent(indent_level: &IndentLevel, ui: &mut egui::Ui) -> egui::Rect {
    let indent = indent_level.to_int() as f32;
    let indent_width = INDENT_GRID_SIZE * indent;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(indent_width, INDENT_GRID_SIZE),
        egui::Sense::hover(),
    );
    rect
}

fn label(text: &str, style: &StyleText, ui: &mut egui::Ui) -> bool {
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
        let mut rich_text = egui::RichText::new(text);
        if *bold {
            rich_text = rich_text.strong();
        }
        if *code {
            rich_text = rich_text.code();
        }
        if *italics {
            rich_text = rich_text.italics();
        }
        if *small {
            rich_text = rich_text.small();
        }
        if *strikethrough {
            rich_text = rich_text.strikethrough();
        }
        if *underline {
            rich_text = rich_text.underline();
        }
        match heading {
            Heading::H1 => rich_text = rich_text.heading().strong(),
            Heading::H2 => rich_text = rich_text.heading(),
            _ => {}
        };
        match heading {
            Heading::None => {}
            _ => {
                line_break = true;
            }
        }
        ui.label(rich_text);
    } else {
        ui.hyperlink_to(text, hyperlink);
    }
    line_break
}

fn new_line(ui: &mut egui::Ui) -> bool {
    ui.allocate_exact_size(egui::vec2(0.0, NEW_LINE_HEIGHT), egui::Sense::hover());
    true
}

fn ordered_listing(
    n: u8,
    text: &str,
    style: &StyleText,
    indent_level: &IndentLevel,
    ui: &mut egui::Ui,
) -> bool {
    indent(indent_level, ui);
    ui.spacing_mut().item_spacing.x = BIG_SPACING_X;
    label(&format!("{}.", n), style, ui);
    ui.spacing_mut().item_spacing.x = NORMAL_SPACING_X;
    label(&text, style, ui);
    true
}

fn quote(text: &str, style: &StyleText, ui: &mut egui::Ui) -> bool {
    ui.spacing_mut().item_spacing.x = BIG_SPACING_X;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(INDENT_GRID_SIZE, INDENT_GRID_SIZE),
        egui::Sense::hover(),
    );
    let rect = rect.expand2(ui.style().spacing.item_spacing * 0.5);
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        (1.0, ui.visuals().weak_text_color()),
    );
    ui.spacing_mut().item_spacing.x = NORMAL_SPACING_X;
    label(&text, style, ui);
    true
}

fn separator(dir: &SeparatorDir, ui: &mut egui::Ui) -> bool {
    match dir {
        SeparatorDir::Horizontal => ui.add(egui::Separator::default().horizontal()),
        SeparatorDir::Vertical => ui.separator(),
    };
    true
}

fn unordered_listing(
    text: &str,
    style: &StyleText,
    indent_level: &IndentLevel,
    ui: &mut egui::Ui,
) -> bool {
    ui.spacing_mut().item_spacing.x = BIG_SPACING_X;
    let rect = indent(indent_level, ui);
    let mark_center = egui::Pos2::new(rect.right(), rect.center().y);
    let mark_size = 4.0;
    let color = ui.visuals().strong_text_color();
    match indent_level {
        IndentLevel::None => {
            ui.painter()
                .circle_filled(mark_center, mark_size / 2.0, color);
        }
        IndentLevel::I1 => {
            ui.painter()
                .circle_stroke(mark_center, mark_size / 2.0, egui::Stroke::new(0.5, color));
        }
        IndentLevel::I2 => {
            ui.painter().rect_filled(
                egui::Rect::from_center_size(mark_center, egui::vec2(mark_size, mark_size)),
                0.0,
                color,
            );
        }
        _ => {
            ui.painter().rect_stroke(
                egui::Rect::from_center_size(mark_center, egui::vec2(mark_size, mark_size)),
                0.0,
                egui::Stroke::new(0.5, color),
            );
        }
    }
    ui.spacing_mut().item_spacing.x = NORMAL_SPACING_X;
    label(&text, style, ui);
    true
}

impl<'a> epi::App for App<'a> {
    fn name(&self) -> &str {
        &self.title
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if self.current_page_idx >= self.pages.len() {
            return;
        }

        let layout = egui::CentralPanel::default().show(ctx, |ui| {
            let initial_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
            let layout = egui::Layout::left_to_right().with_main_wrap(true);
            ui.spacing_mut().item_spacing.x = NORMAL_SPACING_X;
            ui.spacing_mut().item_spacing.y = NORMAL_SPACING_Y;

            ui.allocate_ui_with_layout(initial_size, layout, |outer_ui| {
                let page = &self.pages[self.current_page_idx];
                if let (Mark::Page(transitions), _, transition_idx) = page {
                    for transition in transitions {
                        if let Mark::Transition(order, marks) = transition {
                            for mark in marks {
                                let mut line_break = false;

                                outer_ui.horizontal(|ui| {
                                    if *order > *transition_idx {
                                        ui.set_visible(false);
                                    }
                                    line_break = match mark {
                                        Mark::CodeBlock(code, language) => {
                                            code_block(code, language, ui)
                                        }
                                        Mark::Image(key, title, style) => {
                                            if !self.textures.contains_key(key) {
                                                let texture =
                                                    load_image(Path::new(key), frame).expect(
                                                        &format!("[ERROR] loading image `{}`", key),
                                                    );
                                                self.textures.insert(key.clone(), texture);
                                            };
                                            let tex = self.textures.get(key).expect(&format!(
                                                "[ERROR] loading texture `{}`",
                                                key
                                            ));
                                            image(title, tex, style, ui)
                                        }
                                        Mark::NewLine => new_line(ui),
                                        Mark::Separator(dir) => separator(dir, ui),
                                        Mark::Text(text, style) => match &style.listing {
                                            Listing::Ordered(n, indent) => {
                                                ordered_listing(*n, &text, style, indent, ui)
                                            }
                                            Listing::Unordered(indent) => {
                                                unordered_listing(&text, style, indent, ui)
                                            }
                                            _ => {
                                                if !style.quote {
                                                    label(&text, style, ui)
                                                } else {
                                                    quote(&text, style, ui)
                                                }
                                            }
                                        },
                                        _ => false,
                                    };
                                });

                                if line_break {
                                    outer_ui.end_row();
                                }
                            }
                        }
                    }
                }
            });
        });

        let resp = layout.response.interact(egui::Sense::click());
        if resp.clicked() {
            self.next();
        }

        let is = ctx.input();
        if is.key_released(egui::Key::Escape) {
            frame.quit();
        } else if is.key_released(egui::Key::ArrowRight) || is.key_released(egui::Key::ArrowDown) {
            self.next();
        } else if is.key_released(egui::Key::ArrowLeft) || is.key_released(egui::Key::ArrowUp) {
            self.prev();
        }
    }
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, iter: Parser) -> Self {
        App {
            current_page_idx: 0,
            pages: Parser::into_pages(iter),
            title,
            textures: HashMap::new(),
        }
    }

    fn next(&mut self) {
        let (page, max_transition_idx, transition_idx) = &mut self.pages[self.current_page_idx];
        if let Mark::Page(..) = page {
            if *max_transition_idx > 1 && *transition_idx < *max_transition_idx {
                *transition_idx += 1;
                return;
            }
        }
        if self.current_page_idx < self.pages.len() - 1 {
            self.current_page_idx += 1;
        }
    }

    fn prev(&mut self) {
        let (.., transition_idx) = &mut self.pages[self.current_page_idx];
        if *transition_idx > 0 {
            *transition_idx -= 1;
            return;
        }
        if self.current_page_idx > 0 {
            self.current_page_idx -= 1;
        }
    }
}
