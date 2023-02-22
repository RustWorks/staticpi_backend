#![forbid(unsafe_code)]
#![warn(
    clippy::expect_used,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo,
    clippy::unused_async,
    clippy::unwrap_used,
)]
#![allow(clippy::module_name_repetitions, clippy::doc_markdown)]
// Only allow when debugging
// #![allow(unused, clippy::todo)]

mod api_error;
mod argon;
mod connections;
mod database;
mod emailer;
mod helpers;
mod parse_env;
mod pinger;
mod servers;
mod user_io;

use api_error::ApiError;
use connections::Connections;
use database::connection::ModelConnection;
use parse_env::AppEnv;
use pinger::Pinger;

use servers::{api::ApiServer, token::TokenServer, ws::WsServer, Serve, ServeData, ServerName};

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt};

fn setup_tracing(app_env: &AppEnv) -> Result<(), ApiError> {
    let logfile = tracing_appender::rolling::never(&app_env.location_logs, "staticpi.log");

    let log_fmt = fmt::Layer::default()
        .json()
        .flatten_event(true)
        .with_writer(logfile);

    match tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_file(true)
            .with_line_number(true)
            .with_max_level(app_env.log_level)
            .finish()
            .with(log_fmt),
    ) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{e:?}");
            Err(ApiError::Internal("Unable to start tracing".to_owned()))
        }
    }
}

/// Set all connections to offline, incase application has been incorrectly closed before all ws connections closed correctly
async fn clear_postgres_connections(app_env: &AppEnv) -> Result<(), ApiError> {
    let postgres = database::db_postgres::db_pool(app_env).await?;
    ModelConnection::update_all_offline(&postgres).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    let app_env = parse_env::AppEnv::get_env();
    setup_tracing(&app_env)?;

    clear_postgres_connections(&app_env).await?;
    let connections = Arc::new(Mutex::new(Connections::default()));

    let auth_data = ServeData::new(&app_env, &connections, ServerName::Token).await?;
    tokio::spawn(async move {
        if let Err(e) = TokenServer::serve(auth_data).await {
            error!("{e:?}");
            std::process::exit(1);
        }
    });

    let ws_data = ServeData::new(&app_env, &connections, ServerName::Ws).await?;
    tokio::spawn(async move {
        if let Err(e) = WsServer::serve(ws_data).await {
            error!("{e:?}");
            std::process::exit(1);
        }
    });

    let connections_ping = Arc::clone(&connections);
    let postgres_pinger = database::db_postgres::db_pool(&app_env).await?;
    tokio::spawn(async move {
        Pinger::init(connections_ping, postgres_pinger).await;
    });

    let api_data = ServeData::new(&app_env, &connections, ServerName::Api).await?;
    ApiServer::serve(api_data).await?;
    Ok(())
}
