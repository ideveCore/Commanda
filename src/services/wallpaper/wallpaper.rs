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
use std::sync::Arc;
use tokio::sync::OnceCell;

const BING_BASE_URL: &str = "https://www.bing.com";
const BING_API_URL: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en_US";

#[derive(Clone)]
pub struct WallpaperService {
    client: reqwest::Client,
    cache: Arc<OnceCell<String>>,
}

impl WallpaperService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(OnceCell::new()),
        }
    }

    pub async fn get_image_url(&self) -> Result<&str> {
        self.cache
            .get_or_try_init(|| async {
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
            })
            .await
            .map(|s| s.as_str())
    }
}
