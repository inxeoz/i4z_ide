use anyhow::{anyhow, Result};
use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose, Engine as _};
use image::{ImageFormat, ImageOutputFormat};
use std::io::Cursor;

pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new()
            .map_err(|e| anyhow!("Failed to initialize clipboard: {}", e))?;
        
        Ok(Self { clipboard })
    }

    pub async fn get_image_as_base64(&mut self) -> Result<String> {
        let image_data = self.clipboard
            .get_image()
            .map_err(|e| anyhow!("Failed to get image from clipboard: {}", e))?;

        self.convert_image_to_base64(image_data).await
    }

    async fn convert_image_to_base64(&self, image_data: ImageData<'_>) -> Result<String> {
        let width = image_data.width;
        let height = image_data.height;
        let bytes = image_data.bytes;

        // Convert RGBA bytes to image::RgbaImage
        let img = image::RgbaImage::from_raw(width as u32, height as u32, bytes.into_owned())
            .ok_or_else(|| anyhow!("Failed to create image from clipboard data"))?;

        // Convert to PNG format in memory
        let mut png_data = Vec::new();
        let mut cursor = Cursor::new(&mut png_data);
        
        img.write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| anyhow!("Failed to encode image as PNG: {}", e))?;

        // Encode as base64
        let base64_string = general_purpose::STANDARD.encode(&png_data);
        
        Ok(base64_string)
    }

    pub fn has_image(&mut self) -> bool {
        self.clipboard.get_image().is_ok()
    }

    pub async fn get_text(&mut self) -> Result<String> {
        self.clipboard
            .get_text()
            .map_err(|e| anyhow!("Failed to get text from clipboard: {}", e))
    }

    pub fn set_text(&mut self, text: &str) -> Result<()> {
        self.clipboard
            .set_text(text)
            .map_err(|e| anyhow!("Failed to set clipboard text: {}", e))
    }
}