/* ws.rs
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
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use tracing::info;

use crate::server::SharedState;
use crate::services::mpris::MprisService;

pub fn router() -> Router<SharedState> {
    Router::new().route("/ws", get(ws_handler))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedState>) -> impl IntoResponse {
    let rx = state.event_tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: tokio::sync::broadcast::Receiver<String>) {
    info!("WebSocket connected");

    let mut np = MprisService::get_now_playing().await  ;
    if np.art_url.starts_with("file://") {
        np.art_url = "/api/playing/art".into();
    }
    if let Ok(json) = serde_json::to_string(&np) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    loop {
        tokio::select! {
            Ok(event) = rx.recv() => {
                if socket.send(Message::Text(event.into())).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let cmd = text.trim();
                        if ["play", "pause", "next", "prev"].contains(&cmd) {
                            if let Err(e) = MprisService::send_media_command(cmd).await {
                                tracing::error!("Error while send media command: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    info!("WebSocket disconnected");
}
