use actix_web::{HttpResponse, web};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub async fn process(_db: &Pool<SqliteConnectionManager>, payload: web::Bytes) -> HttpResponse {
    todo!()
}
