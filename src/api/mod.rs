use crate::AppState;
use actix_web::{post, web, HttpRequest, HttpResponse};
use serde::{de::DeserializeOwned, Deserialize};

mod change_message_visibility;
mod create_queue;
mod delete_message;
mod get_queue_attributes;
mod get_queue_url;
mod helpers;
mod list_queues;
mod receive_message;
mod send_message;
mod set_queue_attributes;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct RequestPayload {
    action: String,
}

#[post("/")]
pub async fn post_handler(
    app_state: web::Data<AppState>,
    payload: web::Bytes,
    req: HttpRequest,
) -> HttpResponse {
    let action = match get_action_name(&payload, &req) {
        Some(a) => a,
        None => return HttpResponse::BadRequest().body("Invalid action"),
    };

    let is_json = action.starts_with("AmazonSQS");
    if is_json {
        return HttpResponse::BadRequest().body("JSON is not supported yet");
    }

    match action.to_lowercase().as_str() {
        "amazonsqs.createqueue" | "createqueue" => {
            create_queue::process(app_state.into_inner(), &payload, is_json).await
        }
        "amazonsqs.listqueues" | "listqueues" => {
            list_queues::process(&app_state, &payload, is_json).await
        }
        "amazonsqs.sendmessage" | "sendmessage" => {
            send_message::process(app_state.into_inner(), &payload, is_json).await
        }
        "amazonsqs.receivemessage" | "receivemessage" => {
            receive_message::process(app_state.into_inner(), &payload, is_json).await
        }
        "amazonsqs.deletemessage" | "deletemessage" => {
            delete_message::process(app_state.into_inner(), &payload, is_json).await
        }
        "amazonsqs.changemessagevisibility" | "changemessagevisibility" => {
            change_message_visibility::process(app_state.into_inner(), &payload, is_json).await
        }
        "amazonsqs.getqueueurl" | "getqueueurl" => {
            get_queue_url::process(app_state.into_inner(), &payload, is_json).await
        }
        "amazonsqs.getqueueattributes" | "getqueueattributes" => {
            get_queue_attributes::process(app_state.into_inner(), &payload, is_json).await
        }
        "amazonsqs.setqueueattributes" | "setqueueattributes" => {
            set_queue_attributes::process(app_state.into_inner(), &payload, is_json).await
        }
        _ => HttpResponse::BadRequest().body("Invalid action"),
    }
}

pub(crate) fn struct_from_url_encode<T>(payload: &web::Bytes) -> Result<T, actix_web::Error>
where
    T: DeserializeOwned,
{
    // Convert the Bytes to a &[u8]
    let bytes = payload.as_ref();

    // Now convert the URL-encoded bytes to a struct
    let result: T = serde_urlencoded::from_bytes(bytes).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to parse payload: {}", e))
    })?;

    Ok(result)
}

fn get_action_name(payload: &web::Bytes, req: &HttpRequest) -> Option<String> {
    match req.headers().get("x-amz-target") {
        Some(target) => Some(target.to_str().unwrap().to_string()),
        None => {
            let act = struct_from_url_encode::<RequestPayload>(payload);
            if act.is_err() {
                return None;
            }
            Some(act.unwrap().action.to_string())
        }
    }
}
