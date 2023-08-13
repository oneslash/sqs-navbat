use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::AppState;

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

pub async fn process(app_state: &AppState, payload: &web::Bytes, is_json: bool) -> HttpResponse {
    let params = match get_params(payload, is_json) {
        Some(params) => params,
        None => return HttpResponse::BadRequest().finish(),
    };

    let service = crate::service::queue::Queue::new(&app_state.db_pool, &app_state.host_name);
    let queue_urls = match service.list_queue(
        params.max_results as u32,
        params.queue_name_prefix,
        params.next_token,
    ).await{
        Ok(queue_urls) => queue_urls,
        Err(e) => {
            error!("Failed to list queues: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    let id = Uuid::new_v4();
    let response = ListQueuesResponse {
        list_queues_result: ListQueuesResult {
            queue_url: queue_urls,
        },
        response_metadata: ResponseMetadata {
            request_id: id.to_string(),
        },
    };

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
