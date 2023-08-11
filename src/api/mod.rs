use crate::AppState;
use actix_web::{post, web, HttpRequest, HttpResponse};
use quick_xml::de::from_str;
use serde::{de::DeserializeOwned, Deserialize};

mod create_queue;
mod list_queues;
mod send_message;

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

    return match action.to_lowercase().as_str() {
        "amazonsqs.createqueue" | "createqueue" => {
            create_queue::process(&app_state, &payload, is_json).await
        }
        "amazonsqs.listqueues" | "listqueues" => {
            list_queues::process(&app_state, &payload, is_json).await
        }
        "amazonsqs.sendmessage" => send_message::process(&app_state.db_pool, &payload).await,
        _ => return HttpResponse::BadRequest().body("Invalid action"),
    };
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

/// Create a struct from the payload (web::Bytes)
pub(crate) fn struct_from_xml_payload<T>(payload: &web::Bytes) -> Result<T, actix_web::Error>
where
    T: DeserializeOwned,
{
    let as_str = std::str::from_utf8(payload.as_ref())?;

    // Now convert the XML to a struct
    let result: T = match from_str(&as_str) {
        Ok(r) => r,
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Failed to parse payload: {}",
                e
            )));
        }
    };

    return Ok(result);
}

fn get_action_name(payload: &web::Bytes, req: &HttpRequest) -> Option<String> {
    return match req.headers().get("x-amz-target") {
        Some(target) => Some(target.to_str().unwrap().to_string()),
        None => {
            let act = struct_from_url_encode::<RequestPayload>(&payload);
            if act.is_err() {
                return None;
            }
            Some(act.unwrap().action.to_string())
        }
    };
}
