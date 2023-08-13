use actix_web::{web, HttpResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use super::helpers;
use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SendMessageParams {
    message_body: String,
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
        let re = Regex::new(r"^Attribute\.(\d+)\.(.+)$/i").unwrap();
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

    let msg_id = helpers::generate_random_uuid4();
    let mut writer = app_state.queues.lock().await;
    (*writer)
        .get_mut("myqueue")
        .unwrap()
        .push(crate::queue::Message {
            id: msg_id.clone(),
            message_body: payload.message_body.clone(),
        });

    let response = SendMessageResponse {
        send_message_result: SendMessageResult {
            message_id: msg_id.clone(),
            md5_of_message_body: helpers::compute_md5(payload.message_body.clone().as_str()),
        },
        reponse_metadata: ResponseMetadata {
            request_id: helpers::generate_random_uuid4(),
        },
    };

    return match quick_xml::se::to_string(&response) {
        Ok(resp) => HttpResponse::Ok().body(resp),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to serialize response: {}", e))
        }
    };
}
