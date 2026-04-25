/* wallpaper.rs
 *
 * Copyright 2026 Ideve Core
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::{Context, Result};
use gtk::gio;
use gtk::prelude::SettingsExt;
use std::sync::Arc;
use tokio::sync::OnceCell;

const BING_BASE_URL: &str = "https://www.bing.com";
const BING_API_URL: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en_US";

#[derive(Clone)]
pub struct WallpaperService {
    client: reqwest::Client,
    cache: Arc<OnceCell<Vec<u8>>>,
}

impl WallpaperService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(OnceCell::new()),
        }
    }

    pub async fn get_image(&self) -> Result<&[u8]> {
        self.cache
            .get_or_try_init(|| async {
                match self.fetch_bing_image().await {
                    Ok(bytes) => Ok(bytes),
                    Err(e) => {
                        eprintln!("Bing wallpaper failed ({e}), falling back to system wallpaper");
                        self.fetch_system_wallpaper().await
                    }
                }
            })
            .await
            .map(|v| v.as_slice())
    }

    async fn fetch_bing_image(&self) -> Result<Vec<u8>> {
        let url = self.fetch_bing_url().await?;
        let bytes = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to download Bing wallpaper")?
            .error_for_status()
            .context("Bing returned an error status for the image")?
            .bytes()
            .await
            .context("Failed to read Bing image bytes")?;

        Ok(bytes.to_vec())
    }

    async fn fetch_bing_url(&self) -> Result<String> {
        let json: serde_json::Value = self
            .client
            .get(BING_API_URL)
            .send()
            .await
            .context("Failed to fetch Bing API")?
            .json()
            .await
            .context("Failed to parse Bing API JSON")?;

        let relative_url = json["images"][0]["url"]
            .as_str()
            .context("Missing 'url' field in Bing API response")?;

        Ok(format!("{}{}", BING_BASE_URL, relative_url))
    }

    async fn fetch_system_wallpaper(&self) -> Result<Vec<u8>> {
        let path = tokio::task::spawn_blocking(|| {
            let settings = gio::Settings::new("org.gnome.desktop.background");

            let binding = settings.string("picture-uri");
            let uri = if binding.is_empty() {
                settings.string("picture-uri-dark")
            } else {
                binding
            };

            uri.strip_prefix("file://")
                .map(|s| s.to_owned())
                .unwrap_or_else(|| uri.to_string())
        })
        .await
        .context("Failed to read GSettings")?;

        let bytes = tokio::fs::read(&path)
            .await
            .with_context(|| format!("Failed to read system wallpaper at '{path}'"))?;

        Ok(bytes)
    }
}
