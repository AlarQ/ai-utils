#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::unused_async)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use openai::OpenAIMessage;
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

pub mod langfuse;
pub mod openai;

#[derive(Deserialize)]
struct ChatRequest {
    messages: Vec<OpenAIMessage>,
    conversation_id: Option<String>,
}


#[derive(Serialize)]
struct ChatResponse {
    completion: String,
    completion2: String,
    completion3: String,
    conversation_id: String,
}

// async fn chat(
//     req: web::Json<ChatRequest>,
//     chat_service: web::Data<ChatService>,
//     langfuse_service: web::Data<LangfuseService>,
// ) -> impl Responder {
//     let conversation_id = req
//         .conversation_id
//         .clone()
//         .unwrap_or_else(|| Uuid::new_v4().to_string());

//     let trace =
//         langfuse_service.create_trace(&Uuid::new_v4().to_string(), "Chat", &conversation_id);

//     let mut all_messages = vec![Message {
//         role: "system".to_string(),
//         content: "You are a helpful assistant.".to_string(),
//         name: Some("Alice".to_string()),
//     }];
//     all_messages.extend(req.messages.clone());

//     let mut generated_messages = Vec::new();

//     // Main Completion
//     let main_span = langfuse_service.create_span(&trace, "Main Completion", &all_messages);
//     let main_completion = chat_service
//         .completion(&all_messages, "gpt-4")
//         .await
//         .unwrap();
//     langfuse_service.finalize_span(
//         &main_span,
//         "Main Completion",
//         &all_messages,
//         &main_completion,
//     );
//     let main_message = main_completion.choices[0].message.clone();
//     all_messages.push(main_message.clone());
//     generated_messages.push(main_message);

//     // Secondary Completion
//     let secondary_messages = vec![Message {
//         role: "user".to_string(),
//         content: "Please say 'completion 2'".to_string(),
//         name: None,
//     }];
//     let secondary_span =
//         langfuse_service.create_span(&trace, "Secondary Completion", &secondary_messages);
//     let secondary_completion = chat_service
//         .completion(&secondary_messages, "gpt-4")
//         .await
//         .unwrap();
//     langfuse_service.finalize_span(
//         &secondary_span,
//         "Secondary Completion",
//         &secondary_messages,
//         &secondary_completion,
//     );
//     let secondary_message = secondary_completion.choices[0].message.clone();
//     generated_messages.push(secondary_message);

//     // Third Completion
//     let third_messages = vec![Message {
//         role: "user".to_string(),
//         content: "Please say 'completion 3'".to_string(),
//         name: None,
//     }];
//     let third_span = langfuse_service.create_span(&trace, "Third Completion", &third_messages);
//     let third_completion = chat_service
//         .completion(&third_messages, "gpt-4")
//         .await
//         .unwrap();
//     langfuse_service.finalize_span(
//         &third_span,
//         "Third Completion",
//         &third_messages,
//         &third_completion,
//     );
//     let third_message = third_completion.choices[0].message.clone();
//     generated_messages.push(third_message);

//     // Finalize trace
//     langfuse_service
//         .finalize_trace(&trace, &req.messages, &generated_messages)
//         .await;

//     HttpResponse::Ok().json(ChatResponse {
//         completion: main_completion.choices[0].message.content.clone(),
//         completion2: secondary_completion.choices[0].message.content.clone(),
//         completion3: third_completion.choices[0].message.content.clone(),
//         conversation_id,
//     })
// }

// async fn dd() -> std::io::Result<()> {
//     dotenv().ok();
//     env_logger::init();

//     let chat_service = web::Data::new(ChatService::new());
//     let langfuse_service = web::Data::new(LangfuseService::new());

//     println!("Server running at http://localhost:3000");

//     HttpServer::new(move || {
//         App::new()
//             .app_data(chat_service.clone())
//             .app_data(langfuse_service.clone())
//             .route("/api/chat", web::post().to(chat))
//     })
//     .bind("127.0.0.1:3000")?
//     .run()
//     .await
// }

// #[cfg(test)]
// mod tests {

// }
