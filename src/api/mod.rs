use crate::AppState;
use actix_web::{post, web, HttpRequest, HttpResponse};
use quick_xml::de::from_str;
use serde::{de::DeserializeOwned, Deserialize};

mod create_queue;
mod list_queues;
mod send_message;

#[derive(Deserialize, Debug, Clone)]
struct RequestPayload {
    action: String,
}

#[post("/")]
pub async fn post_handler(
    db: web::Data<AppState>,
    payload: web::Bytes,
    req: HttpRequest,
) -> HttpResponse {
    let action = match get_action_name(&payload, &req) {
        Some(a) => a,
        None => return HttpResponse::BadRequest().body("Invalid action"),
    };

    let is_json = action.starts_with("AmazonSQS");

    return match action.to_lowercase().as_str() {
        "amazonsqs.createqueue" | "createqueue" => {
            create_queue::process(&db.db_pool, payload, is_json).await
        }
        "amazonsqs.listqueues" => list_queues::process(&db.db_pool, payload).await,
        "amazonsqs.sendmessage" => send_message::process(&db.db_pool, payload).await,
        _ => return HttpResponse::BadRequest().body("Invalid action"),
    };
}

/// Create a struct from the payload (web::Bytes)
pub(crate) fn create_stuct_from_payload<T>(payload: &web::Bytes) -> Result<T, actix_web::Error>
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
    let action: &str = match req.headers().get("x-amz-target") {
        Some(target) => target.to_str().unwrap(),
        None => {
            let action: Option<RequestPayload> =
                create_stuct_from_payload(&payload).unwrap_or(None);
            if action.is_none() {
                return None;
            }
            action.unwrap().action.as_str()
        }
    };

    Some(action.to_string())
}
