use actix_web::{web, HttpResponse};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct RequestParams {
    queue_name_prefix: Option<String>,
    #[serde(default = "default_max_results")]
    max_results: i32,
    next_token: Option<String>,
}

/// Default max results is 1000
/// The code is not dead
#[allow(dead_code)]
fn default_max_results() -> i32 {
    1000
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ListQueuesResponse {
    list_queues_result: ListQueuesResult,
    response_metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ListQueuesResult {
    queue_url: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ResponseMetadata {
    request_id: String,
}

pub async fn process(
    db: &Pool<SqliteConnectionManager>,
    payload: &web::Bytes,
    is_json: bool,
) -> HttpResponse {
    let params = match get_params(payload, is_json) {
        Some(params) => params,
        None => return HttpResponse::BadRequest().finish(),
    };

    let conn = match db.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get connection from pool: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut stmt = conn.prepare(r#"SELECT * FROM queues LIMIT ?1"#).unwrap();

    let mut rows = match stmt.query(&[&params.max_results]) {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query database: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut queue_urls: Vec<String> = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let queue_url: String = row.get(1).unwrap();
        queue_urls.push(queue_url);
    }

    let response = ListQueuesResponse { 
        list_queues_result: ListQueuesResult { queue_url: queue_urls },
        response_metadata: ResponseMetadata { request_id: "00000000-0000-0000-0000-000000000000".to_string() },
    };

    error!("Response: {:?}", response);
    let response = match quick_xml::se::to_string(&response) {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to serialize response: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().body(response)
}

#[inline]
fn get_params(payload: &web::Bytes, _is_json: bool) -> Option<RequestParams> {
    let params = match super::struct_from_url_encode::<RequestParams>(payload) {
        Ok(params) => params,
        Err(_) => return None,
    };

    Some(params)
}
