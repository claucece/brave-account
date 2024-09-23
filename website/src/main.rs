use actix_files::Files;
use actix_web::{dev::Server, middleware, web, App, HttpResponse, HttpServer};
use std::net::TcpListener;
use tera::Tera;

pub mod handlers; // new line

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}

pub fn start(listener: TcpListener) -> Result<Server, std::io::Error> {
    let srv = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(TEMPLATES.clone()))
            .wrap(middleware::Logger::default())
            .service(Files::new("/static", "static/").use_last_modified(true))
            .route("/health", web::get().to(HttpResponse::Ok))
            .service(handlers::index)
            .service(handlers::register)
    })
    .listen(listener)?
    .run();

    Ok(srv)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    start(listener)?.await?;
    Ok(())
}
