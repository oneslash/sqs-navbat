use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use clap::Parser;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

mod api;
mod queue;
mod service;

#[derive(clap::Parser, Debug)]
#[command(author, about, version)]
struct CliParams {
    #[clap(short, long, default_value = "127.0.0.1")]
    bind_address: String,
    #[clap(short, long, default_value = "9090")]
    port: u16,
    #[clap(short, long, default_value = "sqlite://database.db")]
    db_url: String,
    #[clap(long, default_value = "http://locahost:9090")]
    host_name: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub host_name: String,
    pub queues: Arc<Mutex<HashMap<String, queue::Queue>>>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli_params = CliParams::parse();

    info!("Creating database connection ...");
    let db_pool = match SqlitePoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(1))
        .connect(&cli_params.db_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Err(anyhow::anyhow!("Failed to connect to database"));
        }
    };

    let queue_list: HashMap<String, queue::Queue> = HashMap::new();
    let state = AppState {
        db_pool,
        host_name: cli_params.host_name,
        queues: Arc::new(Mutex::new(queue_list)),
    };

    info!("Starting server ...");
    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(api::post_handler)
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default())
    })
    .bind((cli_params.bind_address, cli_params.port))?
    .run()
    .await?;
    info!("Server stopped.");

    Ok(())
}

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}
