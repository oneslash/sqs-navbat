use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetQueueAttributesParams {
    queue_url: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetQueueAttributesResponse {
    get_queue_attributes_result: GetQueueAttributesResult,
    response_metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetQueueAttributesResult {
    #[serde(rename = "Attribute")]
    attributes: Vec<AttributeXml>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct AttributeXml {
    name: String,
    value: String,
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
    let params = match super::struct_from_url_encode::<GetQueueAttributesParams>(payload) {
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

    // Collect requested attribute names from AttributeName.N params
    let requested: Vec<String> = params
        .extra
        .iter()
        .filter(|(k, _)| k.starts_with("AttributeName."))
        .map(|(_, v)| v.clone())
        .collect();

    let want_all = requested.is_empty() || requested.contains(&"All".to_string());

    // Get DB-stored attributes
    let service = crate::service::queue::Queue::new(&app_state.db_pool, &app_state.host_name);
    let db_attrs = match service.get_queue_attributes(&queue_name).await {
        Ok(attrs) => attrs,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load queue attributes: {}", e));
        }
    };

    // Get in-memory computed attributes
    let reader = app_state.queues.lock().await;
    let queue = match reader.get(&queue_name) {
        Some(q) => q,
        None => {
            return HttpResponse::BadRequest().body(format!(
                "AWS.SimpleQueueService.NonExistentQueue; Queue: {}",
                queue_name
            ))
        }
    };

    let mut attrs = Vec::new();

    // Always-available computed attributes
    let computed = vec![
        (
            "ApproximateNumberOfMessages",
            queue.approximate_number_of_messages().to_string(),
        ),
        (
            "ApproximateNumberOfMessagesNotVisible",
            queue
                .approximate_number_of_messages_not_visible()
                .to_string(),
        ),
        (
            "VisibilityTimeout",
            queue.default_visibility_timeout.to_string(),
        ),
    ];

    for (name, value) in &computed {
        if want_all || requested.contains(&name.to_string()) {
            attrs.push(AttributeXml {
                name: name.to_string(),
                value: value.clone(),
            });
        }
    }

    for (name, value) in &db_attrs {
        if want_all || requested.contains(name) {
            // Don't duplicate VisibilityTimeout if already added from computed
            if name == "VisibilityTimeout" {
                continue;
            }
            attrs.push(AttributeXml {
                name: name.clone(),
                value: value.clone(),
            });
        }
    }

    let response = GetQueueAttributesResponse {
        get_queue_attributes_result: GetQueueAttributesResult { attributes: attrs },
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
