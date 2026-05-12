use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Health check
    cfg.route("/health", web::get().to(health_check));

    // Register your routes here:
    // cfg.service(
    //     web::scope("/api")
    //         .route("/posts", web::get().to(controllers::post::index))
    //         .route("/posts/{id}", web::get().to(controllers::post::show))
    //         .route("/posts", web::post().to(controllers::post::store))
    //         .route("/posts/{id}", web::delete().to(controllers::post::destroy)),
    // );
}

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "framework": "Orchestra",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
