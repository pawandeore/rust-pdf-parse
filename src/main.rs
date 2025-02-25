use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures_util::stream::StreamExt as _;
use pdf_extract::extract_text;
use std::fs::{self, File};
use std::io::Write;
use uuid::Uuid;

async fn upload_pdf(mut payload: Multipart) -> impl Responder {
    let temp_dir = "uploads/";
    fs::create_dir_all(temp_dir).ok(); // Ensure directory exists

    while let Some(field) = payload.next().await {
        if let Ok(mut field) = field {
            let file_id = Uuid::new_v4().to_string();
            let file_path = format!("{}/{}.pdf", temp_dir, file_id);

            // Save the uploaded file
            if let Ok(mut file) = File::create(&file_path) {
                while let Some(chunk) = field.next().await {
                    if let Ok(data) = chunk {
                        let _ = file.write_all(&data);
                    }
                }
            }

            // Extract text from the PDF
            match extract_text(&file_path) {
                Ok(text) => {
                    let _ = fs::remove_file(&file_path); // Delete after processing
                    return HttpResponse::Ok().json(serde_json::json!({ "text": text }));
                }
                Err(_) => {
                    let _ = fs::remove_file(&file_path); // Cleanup on error
                    return HttpResponse::InternalServerError().body("Error parsing PDF.");
                }
            }
        }
    }

    HttpResponse::BadRequest().body("No file uploaded.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive()) // Enable CORS
            .route("/upload", web::post().to(upload_pdf))
    })
    .bind(("127.0.0.1", 9000))?
    .run()
    .await
}
