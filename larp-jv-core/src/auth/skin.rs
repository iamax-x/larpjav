//! Skin/cape rendering

use crate::error::LarpResult;
use crate::value::Account;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

/// Skin model type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinModel {
    Classic,  // Steve
    Slim,     // Alex
}

/// Skin renderer
pub struct SkinRenderer {
    client: reqwest::Client,
}

impl SkinRenderer {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Download and render skin
    pub async fn render_skin(&self, account: &Account, size: u32) -> LarpResult<DynamicImage> {
        let skin_url = account.skin_url.as_ref()
            .ok_or_else(|| crate::error::LarpError::Other("No skin URL".to_string()))?;

        let model = account.skin_model.as_deref()
            .map(|s| match s {
                "slim" => SkinModel::Slim,
                _ => SkinModel::Classic,
            })
            .unwrap_or(SkinModel::Classic);

        let response = self.client.get(skin_url).send().await?;
        let bytes = response.bytes().await?;

        let skin = image::load_from_memory(&bytes)?;
        Ok(self.render_skin_model(&skin, model, size))
    }

    /// Render skin with model overlay
    fn render_skin_model(&self, skin: &DynamicImage, model: SkinModel, size: u32) -> DynamicImage {
        // Standard skin is 64x64 (or 64x32 for old format)
        // We'll render the head (8x8 at 8,8 for classic, 8x8 at 8,8 for slim)
        let skin_rgba = skin.to_rgba8();

        let mut output = image::RgbaImage::new(size, size);

        // Scale factor
        let scale = size as f32 / 64.0;

        // Render head (front)
        self.copy_region(&skin_rgba, &mut output, 8, 8, 8, 8, 0, 0, size, size, scale);

        // Render head overlay (hat layer) - 8x8 at 40,8
        self.copy_region(&skin_rgba, &mut output, 40, 8, 8, 8, 0, 0, size, size, scale);

        // For slim model, arms are different
        match model {
            SkinModel::Classic => {
                // Body, arms, legs from standard positions
            }
            SkinModel::Slim => {
                // Slim arms are 3px wide instead of 4px
            }
        }

        DynamicImage::ImageRgba8(output)
    }

    fn copy_region(
        &self,
        src: &image::RgbaImage,
        dst: &mut image::RgbaImage,
        sx: u32, sy: u32, sw: u32, sh: u32,
        dx: u32, dy: u32, dw: u32, dh: u32,
        scale: f32,
    ) {
        for y in 0..dh {
            for x in 0..dw {
                let src_x = sx + (x as f32 / scale) as u32;
                let src_y = sy + (y as f32 / scale) as u32;
                if src_x < src.width() && src_y < src.height() {
                    let pixel = src.get_pixel(src_x.min(src.width()-1), src_y.min(src.height()-1));
                    if pixel[3] > 0 { // Only copy non-transparent
                        dst.put_pixel(dx + x, dy + y, *pixel);
                    }
                }
            }
        }
    }

    /// Get skin as PNG bytes
    pub async fn skin_to_png(&self, account: &Account, size: u32) -> LarpResult<Vec<u8>> {
        let img = self.render_skin(account, size).await?;
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Png)?;
        Ok(buf.into_inner())
    }
}

impl Default for SkinRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Skin head renderer for UI (small 8x8 or 16x16)
pub struct SkinHeadRenderer {
    renderer: SkinRenderer,
}

impl SkinHeadRenderer {
    pub fn new() -> Self {
        Self {
            renderer: SkinRenderer::new(),
        }
    }

    pub async fn render_head(&self, account: &Account, size: u32) -> LarpResult<DynamicImage> {
        self.renderer.render_skin(account, size).await
    }
}

impl Default for SkinHeadRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_renderer_creation() {
        let _renderer = SkinRenderer::new();
    }
}