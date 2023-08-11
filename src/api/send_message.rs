use actix_web::{HttpResponse, web};

use crate::AppState;

pub async fn process(app_state: &AppState, payload: &web::Bytes, is_json: bool) -> HttpResponse {
    todo!()
}
