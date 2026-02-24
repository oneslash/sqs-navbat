use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ReceiveMessageParams {
    queue_url: String,
    #[serde(default = "default_max_number")]
    max_number_of_messages: u32,
    #[serde(default)]
    wait_time_seconds: u32,
    visibility_timeout: Option<u32>,
}

fn default_max_number() -> u32 {
    1
}

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
    messages: Vec<MessageXml>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
struct MessageXml {
    message_id: String,
    receipt_handle: String,
    md5_of_body: String,
    body: String,
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
    let params = match super::struct_from_url_encode::<ReceiveMessageParams>(payload) {
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

    let max = params.max_number_of_messages.clamp(1, 10);
    let deadline = tokio::time::Instant::now()
        + tokio::time::Duration::from_secs(params.wait_time_seconds as u64);

    let messages = loop {
        {
            let mut writer = app_state.queues.lock().await;
            match writer.get_mut(&queue_name) {
                Some(queue) => {
                    let received = queue.receive(max, params.visibility_timeout);
                    if !received.is_empty() {
                        break received;
                    }
                }
                None => {
                    return HttpResponse::BadRequest().body(format!(
                        "AWS.SimpleQueueService.NonExistentQueue; Queue: {}",
                        queue_name
                    ))
                }
            }
            // Lock is dropped here before sleeping
        }

        if tokio::time::Instant::now() >= deadline {
            break Vec::new();
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    };

    let xml_messages: Vec<MessageXml> = messages
        .iter()
        .map(|msg| {
            let mut attrs = Vec::new();
            attrs.push(AttributeXml {
                name: "ApproximateReceiveCount".to_string(),
                value: msg.receive_count.to_string(),
            });
            if let Some(first) = msg.first_received_at {
                // Convert monotonic Instant to wall-clock time
                let elapsed_since_first = std::time::Instant::now() - first;
                let first_receive_time = std::time::SystemTime::now() - elapsed_since_first;
                attrs.push(AttributeXml {
                    name: "ApproximateFirstReceiveTimestamp".to_string(),
                    value: first_receive_time
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                        .to_string(),
                });
            }

            MessageXml {
                message_id: msg.id.clone(),
                receipt_handle: msg.receipt_handle.clone().unwrap_or_default(),
                md5_of_body: super::helpers::compute_md5(&msg.message_body),
                body: msg.message_body.clone(),
                attributes: attrs,
            }
        })
        .collect();

    if xml_messages.is_empty() {
        return HttpResponse::Ok()
            .body("<ReceiveMessageResponse><ReceiveMessageResult/></ReceiveMessageResponse>");
    }

    let response = ReceiveMessageResponse {
        receive_message_result: ReceiveMessageResult {
            messages: xml_messages,
        },
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
