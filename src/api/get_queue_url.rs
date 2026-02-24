use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetQueueUrlParams {
    queue_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetQueueUrlResponse {
    get_queue_url_result: GetQueueUrlResult,
    response_metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetQueueUrlResult {
    queue_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ResponseMetadata {
    request_id: String,
}

pub async fn process(
    app_state: Arc<AppState>,
    payload: &web::Bytes,
    _is_json: bool,
) -> HttpResponse {
    let params = match super::struct_from_url_encode::<GetQueueUrlParams>(payload) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Failed to parse payload: {}", e))
        }
    };

    let service = crate::service::queue::Queue::new(&app_state.db_pool, &app_state.host_name);
    match service.queue_exists(&params.queue_name).await {
        Ok(false) => {
            return HttpResponse::BadRequest().body(format!(
                "AWS.SimpleQueueService.NonExistentQueue; Queue: {}",
                params.queue_name
            ));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to query database: {}", e));
        }
        Ok(true) => {}
    }

    let response = GetQueueUrlResponse {
        get_queue_url_result: GetQueueUrlResult {
            queue_url: format!("{}/{}", app_state.host_name, params.queue_name),
        },
        response_metadata: ResponseMetadata {
            request_id: super::helpers::generate_random_uuid4(),
        },
    };

    match quick_xml::se::to_string(&response) {
        Ok(resp) => HttpResponse::Ok().body(resp),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to serialize response: {}", e))
        }
    }
}
