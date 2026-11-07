//! Skin Renderer — port of SkinHeadRenderer.java

use eframe::egui;
use larp_jv_core::{Account, utils::download::download_bytes};
use std::sync::Arc;
use std::collections::HashMap;

pub struct SkinRenderer {
    cache: Arc<std::sync::Mutex<HashMap<String, egui::TextureHandle>>>,
    client: reqwest::Client,
}

impl SkinRenderer {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    pub fn get_skin_texture(&self, account: &Account, ctx: &egui::Context, size: u32) -> Option<egui::TextureHandle> {
        let cache_key = format!("{}_{}", account.id, size);
        
        // Check cache
        if let Some(texture) = self.cache.lock().unwrap().get(&cache_key) {
            return Some(texture.clone());
        }

        // Get skin URL
        let skin_url = account.skin_url.as_ref()?;
        let model = account.skin_model.as_deref().unwrap_or("classic");

        // Download skin
        let client = reqwest::blocking::Client::new();
        let response = client.get(skin_url).send().ok()?;
        let bytes = response.bytes().ok()?;
        
        // Load image
        let image = image::load_from_memory(&bytes).ok()?;
        let image = self.render_skin_model(&image, model, size);
        
        // Create egui texture
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.to_rgba8().as_raw(),
        );
        
        let texture = ctx.load_texture(cache_key.clone(), color_image, egui::TextureOptions::default());
        
        // Cache it
        self.cache.lock().unwrap().insert(cache_key, texture.clone());
        
        Some(texture)
    }

    fn render_skin_model(&self, skin: &image::DynamicImage, model: &str, size: u32) -> image::DynamicImage {
        let skin_rgba = skin.to_rgba8();
        let mut output = image::RgbaImage::new(size, size);
        
        // Head front (8x8 at 8,8)
        self.copy_region(&skin_rgba, &mut output, 8, 8, 8, 8, 0, 0, size, size);
        
        // Head overlay (hat layer) - 8x8 at 40,8
        self.copy_region(&skin_rgba, &mut output, 40, 8, 8, 8, 0, 0, size, size);
        
        image::DynamicImage::ImageRgba8(output)
    }

    fn copy_region(
        &self,
        src: &image::RgbaImage,
        dst: &mut image::RgbaImage,
        sx: u32, sy: u32, sw: u32, sh: u32,
        dx: u32, dy: u32, dw: u32, dh: u32,
    ) {
        let scale_x = sw as f32 / dw as f32;
        let scale_y = sh as f32 / dh as f32;
        
        for y in 0..dh {
            for x in 0..dw {
                let src_x = sx + (x as f32 * scale_x) as u32;
                let src_y = sy + (y as f32 * scale_y) as u32;
                if src_x < src.width() && src_y < src.height() {
                    let pixel = src.get_pixel(src_x.min(src.width()-1), src_y.min(src.height()-1));
                    if pixel[3] > 0 {
                        dst.put_pixel(x, y, *pixel);
                    }
                }
            }
        }
    }
}

impl Default for SkinRenderer {
    fn default() -> Self {
        Self::new()
    }
}