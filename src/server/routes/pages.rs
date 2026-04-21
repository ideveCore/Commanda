/* pages.rs
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

use crate::server::{SharedState, WebAssets};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use minijinja::{context, Environment};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index_handler))
        .route("/{*path}", get(static_handler))
}
async fn static_handler(
    State(state): State<SharedState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let is_static = path.starts_with("static/")
        || path.ends_with(".css")
        || path.ends_with(".js")
        || path.ends_with(".png")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
        || path.ends_with(".woff2");

    if is_static {
        return serve_asset(&path).await;
    }

    // Tenta renderizar como template (ex: "about" → templates/about.html)
    let template_name = if path.ends_with(".html") {
        path.clone()
    } else {
        format!("{path}.html")
    };

    render_template(
        &state.template_env,
        &template_name,
        context! {
            page_title => "Commanda",
        },
    )
    .await
}

async fn index_handler(State(state): State<SharedState>) -> impl IntoResponse {
    render_template(
        &state.template_env,
        "index.html",
        context! {
            page_title => "Commanda — Home",
        },
    )
    .await
}

/// Renderiza um template pelo nome, retornando 404 se não encontrado.
async fn render_template(
    env: &Environment<'static>,
    name: &str,
    ctx: minijinja::Value,
) -> Response {
    match env.get_template(name) {
        Ok(tmpl) => match tmpl.render(ctx) {
            Ok(html) => Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(html))
                .unwrap(),
            Err(e) => {
                eprintln!("Template render error ({name}): {e}");
                serve_500()
            }
        },
        Err(_) => serve_404(),
    }
}

/// Serve um arquivo estático embutido pelo caminho exato.
async fn serve_asset(path: &str) -> Response {
    match WebAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .body(Body::from(content.data))
                .unwrap()
        }
        None => serve_404(),
    }
}

fn serve_404() -> Response {
    let body = WebAssets::get("templates/404.html")
        .and_then(|f| String::from_utf8(f.data.to_vec()).ok())
        .unwrap_or_else(|| "<h1>404 - Não encontrado</h1>".to_string());

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .unwrap()
}

fn serve_500() -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from("500 - Internal Server Error"))
        .unwrap()
}
