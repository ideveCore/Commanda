/* qrcode.rs
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

use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use base64::{engine::general_purpose, Engine as _};
use image::{ImageFormat, Luma};
use qrcode::QrCode;
use std::io::Cursor;

#[derive(Debug, Serialize, Clone)]
pub struct QrCodeData {
    image: String,
}

struct QrCodeCache {
    data: QrCodeData,
    fetched_at: Instant,
}

#[derive(Clone)]
pub struct QrCodeService {
    cache: Arc<Mutex<Option<QrCodeCache>>>,
}

impl QrCodeService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn encode(&self, message: &str) -> Result<QrCodeData> {
        let mut cache = self.cache.lock().await;

        if let Some(ref c) = *cache {
            if c.fetched_at.elapsed() < Duration::from_secs(600) {
                return Ok(c.data.clone());
            }
        }

        let code = QrCode::new(message.as_bytes()).unwrap();

        let image = code.render::<Luma<u8>>().build();

        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageFormat::Png).unwrap();

        let qrcode = QrCodeData {
            image: Self::to_base64(buffer),
        };

        *cache = Some(QrCodeCache {
            data: qrcode.clone(),
            fetched_at: Instant::now(),
        });

        Ok(qrcode)
    }

    fn to_base64(buffer: Cursor<Vec<u8>>) -> String {
        let bytes = buffer.into_inner();
        general_purpose::STANDARD.encode(&bytes)
    }
}
