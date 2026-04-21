/* mod.rs
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

pub mod routes;

use crate::services::wallpaper::WallpaperService;
use axum::Router;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(RustEmbed)]
#[folder = "web/"]
pub struct WebAssets;

#[derive(Clone)]
pub struct AppState {
    pub event_tx: broadcast::Sender<String>,
    pub wallpaper: WallpaperService,
}

pub type SharedState = Arc<AppState>;
pub type EventTx = broadcast::Sender<String>;

pub fn spawn_server(port: u16) -> EventTx {
    let (tx, _) = broadcast::channel::<String>(32);
    let tx_clone = tx.clone();

    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .expect("Falha ao criar runtime Tokio")
            .block_on(run_server(port, tx_clone));
    });

    tx
}

async fn run_server(port: u16, tx: EventTx) {
    let state = Arc::new(AppState {
        event_tx: tx,
        wallpaper: WallpaperService::new(),
    });

    let app = Router::new()
        .merge(routes::all())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Falha ao iniciar o listener");

    info!("Servidor rodando em http://{}", addr);

    axum::serve(listener, app).await.expect("Falha ao servir");
}
