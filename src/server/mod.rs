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

use crate::services::{
    user::SystemUserService, wallpaper::WallpaperService, weather::WeatherService,
};
use axum::Router;
use gtk::gio;
use gtk::gio::prelude::*;
use minijinja::Environment;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
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
    pub weather: WeatherService,
    pub template_env: Arc<Environment<'static>>,
    pub settings: Arc<RwLock<SettingsCache>>,
    pub system_user: SystemUserService,
    pub server_url: String,
}

#[derive(Clone)]
pub struct SettingsCache {
    pub use_ip_location: bool,
    pub location: String,
}

impl SettingsCache {
    fn from_settings(s: &gio::Settings) -> Self {
        Self {
            use_ip_location: s.boolean("use-ip-location"),
            location: s.string("location").to_string(),
        }
    }
}

pub type SharedState = Arc<AppState>;

fn create_template_env() -> Environment<'static> {
    let mut env = Environment::new();

    env.set_loader(|name| {
        let path = format!("templates/{name}");
        Ok(WebAssets::get(&path).and_then(|f| String::from_utf8(f.data.to_vec()).ok()))
    });

    env
}

pub fn spawn_server(port: u16, app_id: &str) -> SharedState {
    let (tx, _) = broadcast::channel::<String>(32);
    let settings = gio::Settings::new(&format!("{}.Weather", app_id));
    let settings_cache = Arc::new(RwLock::new(SettingsCache::from_settings(&settings)));
    let local_ip = local_ip_address::local_ip()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let server_url = format!("http://{}:{}", local_ip, port);

    let state = Arc::new(AppState {
        event_tx: tx,
        wallpaper: WallpaperService::new(),
        weather: WeatherService::new(Arc::clone(&settings_cache)),
        template_env: Arc::new(create_template_env()),
        settings: settings_cache,
        system_user: SystemUserService::new(),
        server_url,
    });

    let state_for_signal = Arc::clone(&state);

    settings.connect_changed(None, move |s, key| {
        tracing::debug!("Setting changed: {}", key);

        if let Ok(mut cache) = state_for_signal.settings.write() {
            *cache = SettingsCache::from_settings(s);
        }
    });

    let state_for_server = Arc::clone(&state);

    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .expect("Falha ao criar runtime Tokio")
            .block_on(run_server(port, state_for_server));
    });

    state
}

async fn run_server(port: u16, state: SharedState) {
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
