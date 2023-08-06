use actix_web::{web, HttpResponse};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub async fn process(db: &Pool<SqliteConnectionManager>, payload: web::Bytes) -> HttpResponse {
    todo!()
}
