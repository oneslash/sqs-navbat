use std::{collections::HashMap, sync::Arc};

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use super::helpers;
use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SendMessageParams {
    message_body: String,
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
        self.attributes = helpers::populate_attributes(self.extra.clone());
    }

    /// Get the attributes as a hashmap
    fn get_attrbutes_hashmap(self) -> HashMap<String, String> {
        helpers::get_attrbutes_hashmap(self.attributes)
    }
}

pub async fn process(app_state: Arc<AppState>, payload: &web::Bytes, is_json: bool) -> HttpResponse {
    let mut payload = match super::struct_from_url_encode::<SendMessageParams>(payload) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Failed to parse payload: {}", e))
        }
    };
    payload.populate_attributes();

    let mut writer = match app_state.queues.lock() {
        Ok(w) => w,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!(
                "Failed to get write lock on queues: {}",
                e
            ))
        }
    };

    writer.get_mut("myqueue")
        .unwrap()
        .push(crate::queue::Message {
            id: "".to_string(),
            message_body: payload.message_body,
        });

    let response = SendMessageResponse {
        send_message_result: SendMessageResult {
            message_id: "123".to_string(),
            md5_of_message_body: "123".to_string(),
        },
        reponse_metadata: ResponseMetadata {
            request_id: "123".to_string(),
        },
    };

    return match quick_xml::se::to_string(&response) {
        Ok(resp) => HttpResponse::Ok().body(resp),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to serialize response: {}", e))
        }
    };
}
