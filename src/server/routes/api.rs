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

use axum::{extract::State, response::Json, routing::get, Router};

use crate::server::SharedState;
use serde_json::{json, Value};

pub fn router() -> Router<SharedState> {
    Router::new().route("/api/status", get(status))
}

async fn status(State(_state): State<SharedState>) -> Json<Value> {
    Json(json!({
      "status": "ok",
      "app": "Commanda",
      "version": env!("CARGO_PKG_VERSION")
    }))
}
