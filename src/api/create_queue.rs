use actix_web::{web, HttpResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use crate::{AppState, api::helpers};

use super::helpers::ParamValues;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueParams {
    queue_name: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,

    #[serde(skip)]
    /// This will be populated when you call populate attributes method
    attributes: Option<Vec<ParamValues>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueAttributes {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueResponse {
    create_queue_result: CreateQueueResult,
    reponse_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueResult {
    queue_url: String,
}

impl CreateQueueParams {
    /// Populate the attributes from the extra hashmap
    fn populate_attributes(&mut self) {
        self.attributes = helpers::populate_attributes(self.extra.clone());
    }

    /// Get the attributes as a hashmap
    fn get_attrbutes_hashmap(self) -> HashMap<String, String> {
        helpers::get_attrbutes_hashmap(self.attributes)
    }
}

/// Create a queue with the given name and attributes
pub async fn process(app_state: &AppState, payload: &web::Bytes, _is_json: bool) -> HttpResponse {
    let mut payload = match super::struct_from_url_encode::<CreateQueueParams>(payload) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Failed to parse payload: {}", e))
        }
    };

    
    payload.populate_attributes();

    let db = &app_state.db_pool;
    let service = crate::service::queue::Queue::new(db, &app_state.host_name);
    let db_result = service.create_queue(crate::service::queue::QueueEntity {
        id: None,
        name: payload.queue_name.clone(),
        attributes: payload.clone().get_attrbutes_hashmap(),
        tag: ("tag_name".to_string(), "tag_value".to_string()),
    });

    info!("Creating queue: {:?}", payload.attributes);
    match db_result {
        Ok(_) => {
            let response = CreateQueueResponse {
                create_queue_result: CreateQueueResult {
                    queue_url: format!("http://localhost:8000/queues/{}", payload.queue_name),
                },
                reponse_metadata: HashMap::new(),
            };

            return match quick_xml::se::to_string(&response) {
                Ok(resp) => HttpResponse::Ok().body(resp),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to serialize response: {}", e)),
            };
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create queue: {}", e))
        }
    }
}
