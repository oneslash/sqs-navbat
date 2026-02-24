use super::helpers;
use crate::AppState;
use actix_web::{web, HttpResponse};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tracing::warn;

/// .fifo - for the FIFO queues
const ATTR_LIST: [&str; 12] = [
    "DelaySeconds",
    "MaximumMessageSize",
    "MessageRetentionPeriod",
    "Policy",
    "ReceiveMessageWaitTimeSeconds",
    "RedrivePolicy",
    "VisibilityTimeout",
    "FifoQueue",
    "ContentBasedDeduplication",
    "KmsMasterKeyId",
    "KmsDataKeyReusePeriodSeconds",
    "SqsManagedSseEnabled",
];

const _ATTR_FIFO: [&str; 4] = [
    "FifoQueue",
    "ContentBasedDeduplication",
    "DeduplicationScope",
    "FifoThroughputLimit",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueParams {
    queue_name: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,

    #[serde(skip)]
    /// This will be populated when you call populate attributes method
    attributes: Option<Vec<helpers::ParamValues>>,

    #[serde(skip)]
    tags: Option<Vec<helpers::ParamValues>>,
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
    fn create_validate_attributes(&mut self) -> anyhow::Result<()> {
        let re = RegexBuilder::new(r"^Attribute\.(\d+)\.(.+)$")
            .case_insensitive(true)
            .build()
            .unwrap();

        self.attributes = helpers::extract_from_extra(re, self.extra.clone());
        if let Some(attrs) = &self.attributes {
            for attr in attrs {
                if !ATTR_LIST.contains(&attr.name.as_str()) {
                    return Err(anyhow::anyhow!("Invalid attribute name: {}", attr.name));
                }
            }
        }

        Ok(())
    }

    fn create_tags(&mut self) {
        let re = RegexBuilder::new(r"^(tags|tag)\.(.+)$")
            .case_insensitive(true)
            .build()
            .unwrap();
        self.tags = helpers::extract_from_extra(re, self.extra.clone());
    }

    /// Get the attributes as a hashmap
    fn get_attrbutes_hashmap(self) -> HashMap<String, String> {
        helpers::get_attrbutes_hashmap(self.attributes)
    }

    fn get_tags_hashmap(self) -> HashMap<String, String> {
        helpers::get_attrbutes_hashmap(self.tags)
    }
}

/// Create a queue with the given name and attributes
pub async fn process(
    app_state: Arc<AppState>,
    payload: &web::Bytes,
    _is_json: bool,
) -> HttpResponse {
    let mut payload = match super::struct_from_url_encode::<CreateQueueParams>(payload) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Failed to parse payload: {}", e))
        }
    };
    match payload.create_validate_attributes() {
        Ok(_) => (),
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Failed to validate attributes: {}", e))
        }
    };
    payload.create_tags();

    let service = crate::service::queue::Queue::new(&app_state.db_pool, &app_state.host_name);
    let db_result = service
        .create_queue(crate::service::queue::QueueEntity {
            id: None,
            name: payload.queue_name.clone(),
            queue_type: "Standard".to_string(),
            attributes: Some(payload.clone().get_attrbutes_hashmap()),
            tags: Some(payload.clone().get_tags_hashmap()),
            created_at: None,
            updated_at: None,
        })
        .await;

    warn!("db_result: {:?}", db_result);
    match db_result {
        Ok(_) => {
            let response = CreateQueueResponse {
                create_queue_result: CreateQueueResult {
                    queue_url: format!("{}/{}", &app_state.host_name, payload.queue_name),
                },
                reponse_metadata: HashMap::new(),
            };

            let visibility_timeout = payload
                .clone()
                .get_attrbutes_hashmap()
                .get("VisibilityTimeout")
                .and_then(|v| v.parse::<u32>().ok());

            let mut writer = app_state.queues.lock().await;
            (*writer).insert(
                payload.queue_name.clone(),
                crate::queue::Queue::new(&payload.queue_name.clone(), vec![], visibility_timeout),
            );

            match quick_xml::se::to_string(&response) {
                Ok(resp) => HttpResponse::Ok().body(resp),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to serialize response: {}", e)),
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to create queue: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// test create_validate_attributes
    #[test]
    fn test_create_validate_attributes() {
        let mut extra = HashMap::new();
        extra.insert("Attribute.1.Name".to_string(), "DelaySeconds".to_string());
        extra.insert("Attribute.1.Value".to_string(), "10".to_string());
        extra.insert(
            "Attribute.2.Name".to_string(),
            "MaximumMessageSize".to_string(),
        );
        extra.insert("Attribute.2.Value".to_string(), "262144".to_string());

        let mut params = CreateQueueParams {
            queue_name: "myqueue".to_string(),
            extra,
            attributes: None,
            tags: None,
        };

        assert!(params.create_validate_attributes().is_ok());
    }

    #[test]
    fn test_create_validate_attributes_fail() {
        let mut extra = HashMap::new();
        extra.insert("Attribute.1.Name".to_string(), "NOT_EXISTS".to_string());
        extra.insert("Attribute.1.Value".to_string(), "10".to_string());
        extra.insert(
            "Attribute.2.Name".to_string(),
            "MaximumMessageSize".to_string(),
        );
        extra.insert("Attribute.2.Value".to_string(), "262144".to_string());

        let mut params = CreateQueueParams {
            queue_name: "myqueue".to_string(),
            extra,
            attributes: None,
            tags: None,
        };

        assert!(params.create_validate_attributes().is_err());
    }
}
