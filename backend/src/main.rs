use actix_web::{get, web, App, HttpServer, Result};
use sha256::digest;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(get_url).service(create))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/{hash}")]
async fn get_url(path: web::Path<String>) -> Result<String> {
    let hash = path.into_inner();
    Ok(format!("URL is {}", hash))
}

#[get("/hash/{url}")]
async fn create(path: web::Path<String>) -> Result<String> {
    let url = path.into_inner();
    let hash = digest(url);
    Ok(format!("Hash is {}", hash))
}
