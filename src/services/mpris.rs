/* mpris.rs
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

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use zbus::zvariant::{OwnedValue, Value};
use zbus::{proxy, Connection, Result as ZResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub player: String,
    pub status: String,
    pub art_url: String,
    pub position: i64,
    pub duration: i64,
    pub rate: f64,
    pub fetched_at: i64,
}

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait MprisPlayer {
    #[zbus(property)]
    fn playback_status(&self) -> ZResult<String>;
    #[zbus(property)]
    fn metadata(&self) -> ZResult<HashMap<String, OwnedValue>>;
    #[zbus(property)]
    fn position(&self) -> ZResult<i64>;
    #[zbus(property)]
    fn rate(&self) -> ZResult<f64>;

    fn next(&self) -> ZResult<()>;
    fn previous(&self) -> ZResult<()>;
    fn pause(&self) -> ZResult<()>;
    fn play(&self) -> ZResult<()>;
}

pub struct MprisService;

impl MprisService {
    fn extract_string(meta: &HashMap<String, OwnedValue>, key: &str) -> String {
        let Some(owned) = meta.get(key) else {
            return String::new();
        };
        match owned.deref() {
            Value::Str(s) => s.to_string(),
            Value::Array(arr) => arr
                .iter()
                .find_map(|v| {
                    if let Value::Str(s) = v {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn now_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    async fn try_get_now_playing() -> ZResult<NowPlaying> {
        let conn = Connection::session().await?;
        let dbus = zbus::fdo::DBusProxy::new(&conn).await?;
        let names = dbus.list_names().await?;

        for name in &names {
            let name_str = name.as_str();
            if !name_str.starts_with("org.mpris.MediaPlayer2.") {
                continue;
            }

            let Ok(proxy) = MprisPlayerProxy::builder(&conn)
                .destination(name_str)?
                .build()
                .await
            else {
                continue;
            };

            let Ok(status) = proxy.playback_status().await else {
                continue;
            };
            if status != "Playing" && status != "Paused" {
                continue;
            }

            let meta = proxy.metadata().await.unwrap_or_default();
            let position = proxy.position().await.unwrap_or(0);
            let rate = proxy.rate().await.unwrap_or(1.0);
            let fetched_at = Self::now_ms();

            let duration = meta
                .get("mpris:length")
                .and_then(|v| i64::try_from(v).ok())
                .unwrap_or(0);

            return Ok(NowPlaying {
                title: Self::extract_string(&meta, "xesam:title"),
                artist: Self::extract_string(&meta, "xesam:artist"),
                album: Self::extract_string(&meta, "xesam:album"),
                player: name_str
                    .trim_start_matches("org.mpris.MediaPlayer2.")
                    .into(),
                status,
                art_url: Self::extract_string(&meta, "mpris:artUrl"),
                position,
                duration,
                rate,
                fetched_at,
            });
        }

        Ok(NowPlaying {
            status: "Stopped".into(),
            ..Default::default()
        })
    }

    pub async fn get_now_playing() -> NowPlaying {
        Self::try_get_now_playing()
            .await
            .unwrap_or_else(|_| NowPlaying {
                status: "Unknown".into(),
                ..Default::default()
            })
    }

    // Listen MPRIS changes
    pub async fn start_monitor(tx: broadcast::Sender<String>) {
        loop {
            match Self::find_active_player_name().await {
                Some(player_name) => {
                    Self::broadcast_now(&tx).await;
                    Self::watch_player_signals(&player_name, &tx).await;
                }
                None => {
                    // No active player found, wait before trying again
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }

    async fn find_active_player_name() -> Option<String> {
        let conn = Connection::session().await.ok()?;
        let dbus = zbus::fdo::DBusProxy::new(&conn).await.ok()?;
        let names = dbus.list_names().await.ok()?;

        for name in &names {
            let name_str = name.as_str();
            if !name_str.starts_with("org.mpris.MediaPlayer2.") {
                continue;
            }
            let proxy = MprisPlayerProxy::builder(&conn)
                .destination(name_str)
                .ok()?
                .build()
                .await
                .ok()?;
            if let Ok(status) = proxy.playback_status().await {
                if status == "Playing" || status == "Paused" {
                    return Some(name_str.to_string());
                }
            }
        }
        None
    }

    async fn watch_player_signals(player_name: &str, tx: &broadcast::Sender<String>) {
        let Ok(conn) = Connection::session().await else {
            return;
        };

        let Ok(proxy) = MprisPlayerProxy::builder(&conn)
            .destination(player_name)
            .unwrap()
            .build()
            .await
        else {
            return;
        };

        // Streams generated automatically by the #[proxy] macro
        let mut status_stream = proxy.receive_playback_status_changed().await;
        let mut meta_stream = proxy.receive_metadata_changed().await;

        loop {
            tokio::select! {
                change = status_stream.next() => {
                    if change.is_none() { break; }
                    Self::broadcast_now(tx).await;
                }
                change = meta_stream.next() => {
                    if change.is_none() { break; }
                    Self::broadcast_now(tx).await;
                }
            }
        }
    }

    async fn broadcast_now(tx: &broadcast::Sender<String>) {
        let mut np = Self::get_now_playing().await;
        if np.art_url.starts_with("file://") {
            np.art_url = "/api/playing/art".into();
        }
        if let Ok(json) = serde_json::to_string(&np) {
            let _ = tx.send(json);
        }
    }

    pub async fn send_media_command(command: &str) -> Result<(), String> {
        let method = match command {
            "next" => "next",
            "prev" => "previous",
            "pause" => "pause",
            "play" => "play",
            other => return Err(format!("unknown command: {other}")),
        };

        let conn = Connection::session().await.map_err(|e| e.to_string())?;
        let dbus = zbus::fdo::DBusProxy::new(&conn)
            .await
            .map_err(|e| e.to_string())?;
        let names = dbus.list_names().await.map_err(|e| e.to_string())?;

        for name in &names {
            let name_str = name.as_str();
            if !name_str.starts_with("org.mpris.MediaPlayer2.") {
                continue;
            }

            let Ok(proxy) = MprisPlayerProxy::builder(&conn)
                .destination(name_str)
                .map_err(|e| e.to_string())?
                .build()
                .await
            else {
                continue;
            };

            let Ok(status) = proxy.playback_status().await else {
                continue;
            };
            if status != "Playing" && status != "Paused" {
                continue;
            }

            let result = match method {
                "next" => proxy.next().await,
                "previous" => proxy.previous().await,
                "pause" => proxy.pause().await,
                "play" => proxy.play().await,
                _ => unreachable!(),
            };

            return result.map_err(|e| e.to_string());
        }

        Err("no active player found".into())
    }
}
