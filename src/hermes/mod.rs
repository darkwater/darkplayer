use std::time::Instant;

use anyhow::{Context as _, Result};
use image::codecs::jpeg::JpegEncoder;
use teloxide::{
    prelude::{Request as _, Requester as _},
    types::{ChatId, InputFile},
};

use crate::hermes::config::Config;

mod config;

#[derive(Debug, Clone)]
pub struct Hermes {
    config: Config,
    bot: teloxide::Bot,
}

impl Hermes {
    pub fn init() -> Result<Self> {
        let config = config::load_config().context("Failed to load config")?;
        let bot = teloxide::Bot::new(&config.token);

        Ok(Self { config, bot })
    }

    pub async fn send_image(
        &self,
        rgba: Vec<u8>,
        width: u32,
        height: u32,
        stride: u32,
    ) -> Result<()> {
        let start = Instant::now(); // takes about 100ms
        let rgb = tokio::task::spawn_blocking(move || {
            let mut rgb = Vec::with_capacity((width * height * 3) as usize);
            for y in 0..height {
                let row_start = (y * stride) as usize;
                let row_end = row_start + (width * 4) as usize;
                let row = &rgba[row_start..row_end];
                for pixel in row.chunks_exact(4) {
                    rgb.extend_from_slice(&pixel[0..3]);
                }
            }
            rgb
        })
        .await
        .unwrap();
        let converting = start.elapsed();

        let start = Instant::now(); // takes about 320ms
        let jpg = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            let mut jpg = Vec::new();
            JpegEncoder::new_with_quality(&mut jpg, 92)
                .encode(&rgb, width, height, image::ExtendedColorType::Rgb8)
                .context("Failed to encode image")?;
            Ok(jpg)
        })
        .await
        .unwrap()?;
        let encoding = start.elapsed();

        let start = Instant::now();
        self.bot
            .send_photo(ChatId(self.config.chat_id), InputFile::memory(jpg))
            .send()
            .await
            .context("Failed to send images")?;
        let sending = start.elapsed();

        log::debug!("Screenshot sent: {converting:?}, {encoding:?}, {sending:?}");

        Ok(())
    }
}
