use eframe::{egui, epi};
use image::{io::Reader, DynamicImage};
use std::path::Path;

pub struct Texture {
    pub id: egui::TextureId,
    pub size: egui::Vec2,
}

pub fn load_image(image_path: &Path, frame: &epi::Frame) -> Result<Texture, image::ImageError> {
    let image = Reader::open(image_path)?.decode()?;
    load_texture(image, frame)
}

fn load_texture(image: DynamicImage, frame: &epi::Frame) -> Result<Texture, image::ImageError> {
    let size = [image.width() as usize, image.height() as usize];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.into_vec();
    let image = epi::Image::from_rgba_unmultiplied(size, &pixels);

    let texture = frame.alloc_texture(image);
    let size = egui::Vec2::new(size[0] as f32, size[1] as f32);

    let texture = Texture { id: texture, size };
    Ok(texture)
}
