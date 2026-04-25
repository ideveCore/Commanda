/* api.rs
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

use axum::{
    extract::{FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use base64::{engine::general_purpose, Engine as _};

use crate::{
    server::SharedState,
    services::{
        mpris::MprisService, qrcode::QrCodeService, user::SystemUserService,
        wallpaper::WallpaperService, weather::WeatherService,
    },
};
use serde_json::{json, Value};

impl FromRef<SharedState> for WallpaperService {
    fn from_ref(state: &SharedState) -> Self {
        state.wallpaper.clone()
    }
}

impl FromRef<SharedState> for WeatherService {
    fn from_ref(state: &SharedState) -> Self {
        state.weather.clone()
    }
}

impl FromRef<SharedState> for SystemUserService {
    fn from_ref(state: &SharedState) -> Self {
        state.system_user.clone()
    }
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/status", get(status))
        .route("/api/wallpaper", get(wallpaper))
        .route("/api/weather", get(weather))
        .route("/api/system_user", get(system_user))
        .route("/api/qrcode", get(qrcode))
        .route("/api/playing/art", get(playing_art))
}

async fn status(State(_state): State<SharedState>) -> Json<Value> {
    Json(json!({
      "status": "ok",
      "app": "Commanda",
      "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn wallpaper(State(svc): State<WallpaperService>) -> impl IntoResponse {
    match svc.get_image().await {
        Ok(bytes) => (
            StatusCode::OK,
            [("content-type", "image/jpeg")],
            bytes.to_vec(),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn weather(State(svc): State<WeatherService>) -> impl IntoResponse {
    match svc.get_weather().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn system_user(State(svc): State<SystemUserService>) -> impl IntoResponse {
    match svc.get_current_user().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn qrcode(State(_state): State<SharedState>) -> impl IntoResponse {
    let svc = QrCodeService::new();

    match svc.encode(&_state.server_url).await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn playing_art() -> impl IntoResponse {
    let np = MprisService::get_now_playing().await;
    if np.art_url.starts_with("file://") {
        let path = np.art_url.trim_start_matches("file://");

        let mut file_bytes = tokio::fs::read(path).await;

        if file_bytes.is_err() {
            if let Ok(mut entries) = tokio::fs::read_dir("/proc").await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let pid = entry.file_name();
                    if pid.to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
                        let proc_path = format!("/proc/{}/root{}", pid.to_string_lossy(), path);
                        if let Ok(bytes) = tokio::fs::read(&proc_path).await {
                            file_bytes = Ok(bytes);
                            break;
                        }
                    }
                }
            }
        }

        match file_bytes {
            Ok(bytes) => {
                let data = json!({
                  "image": general_purpose::STANDARD.encode(bytes),
                });
                (StatusCode::OK, Json(data)).into_response()
            }
            Err(_) => (StatusCode::NOT_FOUND, "Art not found").into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "No local art available").into_response()
    }
}
