/* user.rs
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
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use zbus::Connection;

#[derive(Debug)]
struct UserCache {
    data: SystemUser,
    fetched_at: Instant,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemUser {
    pub username: String,
    pub real_name: String,
    pub icon_file: String,
    pub account_type: i32,
    pub uid: u64,
}

#[derive(Clone, Debug)]
pub struct SystemUserService {
    cache: Arc<Mutex<Option<UserCache>>>,
}

impl SystemUserService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_current_user(&self) -> Result<SystemUser> {
        let mut cache = self.cache.lock().await;

        if let Some(ref c) = *cache {
            if c.fetched_at.elapsed() < Duration::from_secs(600) {
                return Ok(c.data.clone());
            }
        }

        let uid = unsafe { libc::getuid() } as u64;

        let conn = Connection::system()
            .await
            .context("Failed to connect to D-Bus system bus")?;

        let accounts_proxy = zbus::Proxy::new(
            &conn,
            "org.freedesktop.Accounts",
            "/org/freedesktop/Accounts",
            "org.freedesktop.Accounts",
        )
        .await?;

        let user_path: zbus::zvariant::OwnedObjectPath = accounts_proxy
            .call("FindUserById", &(uid as i64))
            .await
            .context("Failed to find user by UID")?;

        let user_proxy = zbus::Proxy::new(
            &conn,
            "org.freedesktop.Accounts",
            user_path.as_str(),
            "org.freedesktop.Accounts.User",
        )
        .await?;

        let username: String = user_proxy.get_property("UserName").await?;
        let real_name: String = user_proxy.get_property("RealName").await?;
        let icon_file: String = user_proxy.get_property("IconFile").await?;
        let account_type: i32 = user_proxy.get_property("AccountType").await?;

        let user = SystemUser {
            username,
            real_name,
            icon_file,
            account_type,
            uid,
        };

        *cache = Some(UserCache {
            data: user.clone(),
            fetched_at: Instant::now(),
        });

        Ok(user)
    }
}
