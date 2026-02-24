use actix_web::{web, HttpResponse};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use super::helpers;
use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SetQueueAttributesParams {
    queue_url: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct SetQueueAttributesResponse {
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
    let params = match super::struct_from_url_encode::<SetQueueAttributesParams>(payload) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Failed to parse payload: {}", e))
        }
    };

    let queue_name = match helpers::extract_queue_name_from_url(&params.queue_url) {
        Some(name) => name,
        None => {
            return HttpResponse::BadRequest()
                .body("Invalid QueueUrl: could not extract queue name")
        }
    };

    // Parse Attribute.N.Name / Attribute.N.Value pairs
    let re = RegexBuilder::new(r"^Attribute\.(\d+)\.(.+)$")
        .case_insensitive(true)
        .build()
        .unwrap();
    let param_values = helpers::extract_from_extra(re, params.extra);
    let attrs = helpers::get_attrbutes_hashmap(param_values);

    if attrs.is_empty() {
        return HttpResponse::BadRequest().body("No attributes provided");
    }

    // Check queue exists
    {
        let reader = app_state.queues.lock().await;
        if !reader.contains_key(&queue_name) {
            return HttpResponse::BadRequest().body(format!(
                "AWS.SimpleQueueService.NonExistentQueue; Queue: {}",
                queue_name
            ));
        }
    }

    // Update in-memory VisibilityTimeout if provided
    if let Some(vt) = attrs.get("VisibilityTimeout") {
        if let Ok(timeout) = vt.parse::<u32>() {
            let mut writer = app_state.queues.lock().await;
            if let Some(queue) = writer.get_mut(&queue_name) {
                queue.default_visibility_timeout = timeout;
            }
        }
    }

    // Persist to DB
    let service = crate::service::queue::Queue::new(&app_state.db_pool, &app_state.host_name);
    if let Err(e) = service.set_queue_attributes(&queue_name, attrs).await {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to set attributes: {}", e));
    }

    let response = SetQueueAttributesResponse {
        response_metadata: ResponseMetadata {
            request_id: helpers::generate_random_uuid4(),
        },
    };

    match quick_xml::se::to_string(&response) {
        Ok(resp) => HttpResponse::Ok().body(resp),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to serialize response: {}", e))
        }
    }
}
