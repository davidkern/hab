mod config;

use std::sync::Arc;

use anyhow::Result;
use axum::{response::Json, routing::get, Router, extract::State};
use influxdb2::FromDataPoint;
use influxdb2_structmap::FromMap;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    let rt = Runtime::new()?;
    rt.block_on(async move {
        pretty_env_logger::init();

        let config = Arc::new(config::Config::load()?);
        let bind_address = config.bind_address.to_owned();

        let app = Router::new()
            .route("/", get(v0_data))
            .with_state(config);

        axum::Server::bind(&bind_address.parse()?)
            .serve(app.into_make_service())
            .await?;

        log::debug!("exiting");
        Ok(())
    })
}

#[derive(Clone, Default, FromDataPoint, Serialize, Debug)]
struct V0Mppt {
    battery_current: f64,
    battery_voltage: f64,
    error: String,
    maximum_power_today: f64,
    panel_power: f64,
    panel_voltage: f64,
    state: String,
    yield_today: f64,
    yield_total: f64,
}

#[derive(Clone, Default, FromDataPoint, Serialize, Debug)]
struct V0Dc {
    charger_current: f64,
    charger_watts: f64,
    inverter_current: f64,
    inverter_frequency: f64,
    inverter_watts: f64,
    voltage: f64,
}

#[derive(Clone, Default, FromDataPoint, Serialize, Debug)]
struct V0Ac {
    bf_factor: f64,
    inverter_current: f64,
    inverter_factor: f64,
    inverter_voltage: f64,
    inverter_watts: f64,
    mains_current: f64,
    mains_frequency: f64,
    mains_voltage: f64,
    mains_watts: f64,
    state: String,
}

async fn query_measurement<T: FromMap + Clone>(db: &influxdb2::Client, name: &str) -> Option<T> {
    let q = influxdb2::models::Query::new(format!(r#"
        from(bucket: "hab")
        |> range(start: -60s)
        |> filter(fn: (r) => r._measurement == "{}")
        |> last()
    "#, name));

    db.query(Some(q)).await.map_or_else(|e| panic!("{}", e.to_string()), |r| r.get(0).cloned())
}

async fn v0_data(
    State(config): State<Arc<config::Config>>,
) -> Json<Value> {
    // All of this is hardcoded for expediency

    let db = influxdb2::Client::new(&config.influxdb_url, &config.influxdb_org, &config.influxdb_token);

    let mppt_lil: Option<V0Mppt> = query_measurement(&db, "mppt_lil").await;
    let mppt_big: Option<V0Mppt> = query_measurement(&db, "mppt_big").await;
    let mppt_ext: Option<V0Mppt> = query_measurement(&db, "mppt_ext").await;
    let inverter_dc: Option<V0Dc> = query_measurement(&db, "dc").await;
    let inverter_ac: Option<V0Ac> = query_measurement(&db, "ac").await;

    Json(json!({
        "mppt": {
            "lil": mppt_lil,
            "big": mppt_big,
            "ext": mppt_ext,
        },
        "inverter": {
            "primary": {
                "ac": inverter_ac,
                "dc": inverter_dc,
            },
        },
        "imu": {
            "primary": null,
        }
    }))
}
