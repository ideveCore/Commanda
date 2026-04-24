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

use crate::{
    server::SharedState,
    services::{user::SystemUserService, wallpaper::WallpaperService, weather::WeatherService},
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
