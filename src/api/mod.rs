use actix_web::{post, web, HttpResponse};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Crate imports
use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PostHandlerQueryParams {
    action: String,
    queue_name: String,
    version: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,

    #[serde(skip)]
    /// This will be populated when you call populate attributes method
    attributes: Option<Vec<CreateQueueAttributes>>,
}

impl PostHandlerQueryParams {
    /// Populate the attributes from the extra hashmap
    fn populate_attributes(&mut self) {
        // Let's live dangerously and unwrap this
        let re = Regex::new(r"Attribute\.(\d+)\.(.+)$").unwrap();

        let mut attrs = Vec::new();

        for (key, value) in self.extra.iter() {
            if let Some(caps) = re.captures(key) {
                let index = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
                let attr_name = caps.get(2).unwrap().as_str().to_string();

                if attrs.len() <= index {
                    attrs.resize(index + 1, CreateQueueAttributes {
                        name: "".to_string(),
                        value: "".to_string(),
                    });
                }

                match attr_name.as_str() {
                    "Name" => attrs[index].name = value.to_string(),
                    "Value" => attrs[index].value = value.to_string(),
                    _ => (),
                }
            }
        }
        attrs.remove(0);

        self.attributes = Some(attrs);
    }

    /// Get the attributes as a hashmap
    fn get_attrbutes_hashmap(self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        if let Some(attrs) = self.attributes {
            for attr in attrs {
                map.insert(attr.name, attr.value);
            }
        }

        return map; 
    }
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

#[post("/")]
pub async fn post_handler(
    db: web::Data<AppState>,
    payload: web::Form<PostHandlerQueryParams>,
) -> HttpResponse {
    info!("Received request: {:?}", payload);

    let mut payload = payload.into_inner();
    payload.populate_attributes();

    return match payload.action.to_lowercase().as_str() {
        "createqueue" => create_queue(&db.db_pool, &payload).await,
        _ => return HttpResponse::BadRequest().body("Invalid action"),
    };
}

/// Create a queue with the given name and attributes
pub async fn create_queue(
    db: &Pool<SqliteConnectionManager>,
    params: &PostHandlerQueryParams,
) -> HttpResponse {
    println!("Creating queue: {:?}", params.attributes);
    let service = crate::service::Queue::new(db);

    let db_result = service.create_queue(crate::service::QueueEntity {
        id: 0,
        name: params.queue_name.clone(),
        attributes: params.clone().get_attrbutes_hashmap(),
        tag: ("tag_name".to_string(), "tag_value".to_string()),
    });

    match db_result {
        Ok(_) => {
            let response = CreateQueueResponse {
                create_queue_result: CreateQueueResult {
                    queue_url: format!("http://localhost:8000/queues/{}", params.queue_name),
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
