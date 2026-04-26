/* weather.rs
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

use crate::server::SettingsCache;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct IpLocation {
    pub lat: f64,
    pub lon: f64,
    pub city: String,
    #[serde(rename = "regionName")]
    pub region: String,
    pub country: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenMeteoResponse {
    current_weather: CurrentWeather,
}

#[derive(Debug, Deserialize)]
pub struct CurrentWeather {
    temperature: f64,
    windspeed: f64,
    weathercode: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct WeatherData {
    city: String,
    region: String,
    country: String,
    temperature: f64,
    windspeed: f64,
    weathercondition: String,
}

#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<LocationInfo>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocationInfo {
    pub id: i32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub elevation: f64,
    #[serde(default)]
    pub feature_code: String,
    #[serde(default)]
    pub country_code: String,
    #[serde(default)]
    pub admin1_id: i32,
    #[serde(default)]
    pub admin2_id: i32,
    #[serde(default)]
    pub timezone: String,
    #[serde(default)]
    pub population: i32,
    #[serde(default)]
    pub country_id: i32,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub admin1: String,
    #[serde(default)]
    pub admin2: String,
}

struct WeatherCache {
    data: WeatherData,
    fetched_at: Instant,
}

#[derive(Clone)]
pub struct WeatherService {
    client: reqwest::Client,
    cache: Arc<Mutex<Option<WeatherCache>>>,
    settings: SettingsCache,
}

impl WeatherService {
    pub fn new(settings: Arc<RwLock<SettingsCache>>) -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(None)),
            settings: settings.read().unwrap().clone(),
        }
    }

    pub async fn get_weather(&self) -> Result<WeatherData> {
        let mut cache = self.cache.lock().await;

        if let Some(ref c) = *cache {
            if c.fetched_at.elapsed() < Duration::from_secs(600) {
                return Ok(c.data.clone());
            }
        }

        let loc = self
            .get_my_location()
            .await
            .context("Failed fetch location")?;

        let url = format!("https://api.open-meteo.com/v1/forecast?latitude={:.4}&longitude={:.4}&current_weather=true", loc.lat, loc.lon);

        let data = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed fetch weather")?
            .json::<OpenMeteoResponse>()
            .await
            .context("Failed parse weather")?;

        let weather = WeatherData {
            city: loc.city,
            region: loc.region,
            country: loc.country,
            temperature: data.current_weather.temperature,
            windspeed: data.current_weather.windspeed,
            weathercondition: Self::get_weather_condition(data.current_weather.weathercode),
        };

        *cache = Some(WeatherCache {
            data: weather.clone(),
            fetched_at: Instant::now(),
        });

        Ok(weather)
    }

    pub async fn get_locations(&self, name: &str) -> Result<Vec<LocationInfo>> {
        let url = format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=10&language=pt&format=json", name);

        let data = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed fetch location")?
            .json::<GeocodingResponse>()
            .await
            .context("Failed parse location")?;

        Ok(data.results.unwrap_or_default())
    }

    async fn get_my_location(&self) -> Result<IpLocation> {
        if self.settings.use_ip_location {
            let loc = reqwest::get("http://ip-api.com/json/")
                .await
                .context("Failed to get location")?
                .json::<IpLocation>()
                .await
                .context("Failed to parse location")?;

            Ok(loc)
        } else {
            let info: LocationInfo = serde_json::from_str(self.settings.location.as_str())
                .context("Failed to parse location")?;

            Ok(IpLocation {
                lat: info.latitude,
                lon: info.longitude,
                city: info.name.clone(),
                region: info.admin1.clone(),
                country: info.country.clone(),
            })
        }
    }

    fn get_weather_condition(code: i32) -> String {
        match code {
            0 => "clear_sky".to_string(),
            1 => "mainly_clear".to_string(),
            2 => "partly_cloudy".to_string(),
            3 => "overcast".to_string(),
            45 => "fog".to_string(),
            48 => "depositing_rime_fog".to_string(),
            51 => "light_drizzle".to_string(),
            53 => "moderate_drizzle".to_string(),
            55 => "dense_drizzle".to_string(),
            56 => "light_freezing_drizzle".to_string(),
            57 => "dense_freezing_drizzle".to_string(),
            61 => "slight_rain".to_string(),
            63 => "moderate_rain".to_string(),
            65 => "heavy_rain".to_string(),
            66 => "light_freezing_rain".to_string(),
            67 => "heavy_freezing_rain".to_string(),
            71 => "slight_snow_fall".to_string(),
            73 => "moderate_snow_fall".to_string(),
            75 => "heavy_snow_fall".to_string(),
            77 => "snow_grains".to_string(),
            80 => "slight_rain_showers".to_string(),
            81 => "moderate_rain_showers".to_string(),
            82 => "violent_rain_showers".to_string(),
            85 => "slight_snow_showers".to_string(),
            86 => "heavy_snow_showers".to_string(),
            95 => "thunderstorm".to_string(),
            96 => "thunderstorm_with_slight_hail".to_string(),
            99 => "thunderstorm_with_heavy_hail".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

impl std::fmt::Debug for WeatherService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeatherService").finish()
    }
}
