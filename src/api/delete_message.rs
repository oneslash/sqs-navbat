use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DeleteMessageParams {
    queue_url: String,
    receipt_handle: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct DeleteMessageResponse {
    response_metadata: ResponseMetadata,
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
    let params = match super::struct_from_url_encode::<DeleteMessageParams>(payload) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Failed to parse payload: {}", e))
        }
    };

    let queue_name = match super::helpers::extract_queue_name_from_url(&params.queue_url) {
        Some(name) => name,
        None => {
            return HttpResponse::BadRequest()
                .body("Invalid QueueUrl: could not extract queue name")
        }
    };

    let mut writer = app_state.queues.lock().await;
    match writer.get_mut(&queue_name) {
        Some(queue) => {
            if !queue.delete_by_receipt_handle(&params.receipt_handle) {
                return HttpResponse::BadRequest().body(
                    "ReceiptHandleIsInvalid; The input receipt handle is not a valid receipt handle.",
                );
            }
        }
        None => {
            return HttpResponse::BadRequest().body(format!(
                "AWS.SimpleQueueService.NonExistentQueue; Queue: {}",
                queue_name
            ))
        }
    }

    let response = DeleteMessageResponse {
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
