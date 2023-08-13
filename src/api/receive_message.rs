use actix_web::{web, HttpResponse};
use serde::Serialize;
use tracing::error;
use std::{
    collections::HashMap,
    sync::Arc,
};

use crate::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ReceiveMessageResponse {
    receive_message_result: ReceiveMessageResult,
    response_metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ReceiveMessageResult {
    #[serde(rename = "Message")]
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
struct Message {
    message_id: String,
    receipt_handle: String,
    md5_of_body: String,
    body: String,
    attributes: HashMap<String, String>,
    md5_of_message_attributes: String,
    message_attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ResponseMetadata {
    request_id: String,
}

pub async fn process(
    app_state: Arc<AppState>,
    _payload: &web::Bytes,
    _is_json: bool,
) -> HttpResponse {
    let mut reader = app_state.queues.lock().await;

    let message = (*reader).get_mut("myqueue").unwrap().pop();
    let messages = match message {
        Some(msg) => vec![
            Message {
                message_id: msg.id.to_string(),
                receipt_handle: "".to_string(),
                md5_of_body: crate::api::helpers::compute_md5(msg.message_body.as_str()),
                body: msg.message_body.to_string(),
                attributes: HashMap::new(),
                md5_of_message_attributes: "".to_string(),
                message_attributes: HashMap::new(),
            }
        ],
        None => {
            error!("No messages in queue");
            return HttpResponse::Ok().body("<ReceiveMessageResponse></ReceiveMessageResponse>");
        }
    };

    let response = ReceiveMessageResponse {
        receive_message_result: ReceiveMessageResult {
            messages,
        },
        response_metadata: ResponseMetadata {
            request_id: crate::api::helpers::generate_random_uuid4(),
        },
    };

    return match quick_xml::se::to_string(&response) {
        Ok(resp) => HttpResponse::Ok().body(resp),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to serialize response: {}", e))
        }
    };
}
