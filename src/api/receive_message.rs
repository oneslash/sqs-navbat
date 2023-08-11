use actix_web::{web, HttpResponse};
use serde::Serialize;
use tracing::error;
use std::{collections::HashMap, sync::Arc};

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
    payload: &web::Bytes,
    is_json: bool,
) -> HttpResponse {
    let mut reader = match app_state.queues.lock() {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to get read lock on queues: {}", e))
        }
    };

    let message = reader.get_mut("myqueue").unwrap().pop();

    error!("Message: {:?}", message);
    let response = ReceiveMessageResponse {
        receive_message_result: ReceiveMessageResult {
            messages: vec![Message {
                message_id: "".to_string(),
                receipt_handle: "".to_string(),
                md5_of_body: "".to_string(),
                body: message.unwrap().message_body,
                attributes: HashMap::new(),
                md5_of_message_attributes: "".to_string(),
                message_attributes: HashMap::new(),
            }],
        },
        response_metadata: ResponseMetadata {
            request_id: "".to_string(),
        },
    };

    

    return match quick_xml::se::to_string(&response) {
        Ok(resp) => HttpResponse::Ok().body(resp),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to serialize response: {}", e))
        }
    };
}
