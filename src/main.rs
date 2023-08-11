use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use clap::Parser;
use r2d2_sqlite::SqliteConnectionManager;
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info};
use tokio::sync::Mutex;

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
    #[clap(short, long, default_value = "database.db")]
    db_path: String,
    #[clap(long, default_value = "http://locahost")]
    host_name: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: r2d2::Pool<SqliteConnectionManager>,
    pub host_name: String,
    pub queues: Arc<Mutex<HashMap<String, queue::Queue>>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let cli_params = CliParams::parse();
    let host_name = format!("{}:{}", cli_params.host_name, cli_params.port);

    info!("Creating database connection ...");
    let conn_manager = SqliteConnectionManager::file(cli_params.db_path);
    let pool = match r2d2::Pool::new(conn_manager) {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to create database connection pool: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create database connection pool",
            ));
        }
    };

    let queue_list: HashMap<String, queue::Queue> = HashMap::new();
    let state = AppState {
        db_pool: pool,
        host_name,
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
