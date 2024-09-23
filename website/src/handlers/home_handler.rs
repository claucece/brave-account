use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

// Struct to capture the form data
#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    email: String,
    password: String,
}

#[get("/")]
pub async fn index(templates: web::Data<tera::Tera>) -> impl Responder {
    let mut context = tera::Context::new();

    match templates.render("register.html", &context) {
        Ok(s) => HttpResponse::Ok().content_type("text/html").body(s),
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError()
                .content_type("text/html")
                .body("<p>Something went wrong!</p>")
        }
    }
}

#[post("/register")]
pub async fn register(form: web::Form<RegisterForm>) -> impl Responder {
    println!("Email: {}, Password: {}", form.email, form.password);

    // Handle form processing (e.g., save to database, authentication, etc.)
    // Return response
    HttpResponse::Ok().body(format!(
        "Received email: {}, password: {}",
        form.email, form.password
    ))
}
