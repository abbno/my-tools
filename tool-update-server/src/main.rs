mod models;
mod storage;
mod handlers;

use actix_web::{App, HttpServer, web};
use actix_files::Files;
use crate::handlers::AppState;

/// Parse -port argument from command line arguments
/// Returns the port number if found, None otherwise
fn parse_port_arg(args: &[String]) -> Option<u16> {
    for i in 0..args.len() {
        if args[i] == "-port" && i + 1 < args.len() {
            return args[i + 1].parse::<u16>().ok();
        }
    }
    None
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Parse port from command line arguments
    let args: Vec<String> = std::env::args().collect();
    let port = parse_port_arg(&args).unwrap_or(39000);

    // Get data directory (default: ./data relative to executable)
    let exe_dir = std::env::current_exe()
        .expect("Cannot get executable path")
        .parent()
        .expect("Cannot get executable directory")
        .to_path_buf();

    let data_dir = exe_dir.join("data");

    // Initialize storage
    let storage = storage::Storage::new(data_dir.clone());
    storage.init()?;

    println!("Tool Update Server starting...");
    println!("Data directory: {}", data_dir.display());
    println!("Server running at http://0.0.0.0:{}", port);

    // Create app state
    let app_state = web::Data::new(AppState { storage });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Static files
            .service(Files::new("/static", exe_dir.join("static")).show_files_listing())
            // Page routes (HTML) - order matters: specific routes before parameterized routes
            .route("/", web::get().to(handlers::index_page))
            .route("/app/new", web::get().to(handlers::new_app_page))
            .route("/app/{app_id}", web::get().to(handlers::app_detail_page))
            // Form routes (HTML form submission)
            .route("/api/apps/form", web::post().to(handlers::create_app_form))
            .route("/api/apps/{app_id}/versions/form", web::post().to(handlers::upload_version_form))
            // API routes for client
            .route("/api/version/{app_id}", web::get().to(handlers::get_latest_version))
            .route("/api/version/{app_id}/{version}", web::get().to(handlers::get_version))
            .route("/download/{app_id}/{filename}", web::get().to(handlers::download_file))
            // API routes for management (JSON)
            .route("/api/apps", web::get().to(handlers::get_apps))
            .route("/api/apps", web::post().to(handlers::create_app))
            .route("/api/apps/{app_id}/versions", web::get().to(handlers::get_app_versions))
            .route("/api/apps/{app_id}/versions", web::post().to(handlers::upload_version))
            .route("/api/apps/{app_id}/versions/{version}", web::delete().to(handlers::delete_version))
            // Health check
            .route("/health", web::get().to(|| async { "OK" }))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}