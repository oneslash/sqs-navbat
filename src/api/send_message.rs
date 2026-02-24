use actix_web::{web, HttpResponse};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use super::helpers;
use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SendMessageParams {
    queue_url: String,
    message_body: String,
    #[allow(dead_code)]
    delay_seconds: Option<i32>,
    #[serde(flatten)]
    extra: HashMap<String, String>,

    #[serde(skip)]
    /// This will be populated when you call populate attributes method
    attributes: Option<Vec<helpers::ParamValues>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendMessageResponse {
    send_message_result: SendMessageResult,
    reponse_metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendMessageResult {
    message_id: String,
    md5_of_message_body: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ResponseMetadata {
    request_id: String,
}

impl SendMessageParams {
    /// Populate the attributes from the extra hashmap
    fn populate_attributes(&mut self) {
        let re = RegexBuilder::new(r"^Attribute\.(\d+)\.(.+)$")
            .case_insensitive(true)
            .build()
            .unwrap();

        self.attributes = helpers::extract_from_extra(re, self.extra.clone());
    }
}

pub async fn process(
    app_state: Arc<AppState>,
    payload: &web::Bytes,
    _is_json: bool,
) -> HttpResponse {
    let mut payload = match super::struct_from_url_encode::<SendMessageParams>(payload) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Failed to parse payload: {}", e))
        }
    };
    payload.populate_attributes();

    let queue_name = match helpers::extract_queue_name_from_url(&payload.queue_url) {
        Some(name) => name,
        None => {
            return HttpResponse::BadRequest()
                .body("Invalid QueueUrl: could not extract queue name")
        }
    };

    let msg_id = helpers::generate_random_uuid4();
    let mut writer = app_state.queues.lock().await;
    match writer.get_mut(&queue_name) {
        Some(queue) => {
            queue.push(crate::queue::Message::new(
                msg_id.clone(),
                payload.message_body.clone(),
            ));
        }
        None => {
            return HttpResponse::BadRequest().body(format!(
                "AWS.SimpleQueueService.NonExistentQueue; see the SQS docs. Queue: {}",
                queue_name
            ))
        }
    }

    let response = SendMessageResponse {
        send_message_result: SendMessageResult {
            message_id: msg_id.clone(),
            md5_of_message_body: helpers::compute_md5(payload.message_body.as_str()),
        },
        reponse_metadata: ResponseMetadata {
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
