use actix_files::NamedFile;
use actix_web::{get, post, web, HttpResponse, Responder};
use base64::encode;
use opaque_ke::{ciphersuite::CipherSuite, ksf::Ksf, Ristretto255};
use opaque_ke::{
    ClientRegistration, ClientRegistrationFinishParameters, CredentialFinalization,
    CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload,
    ServerRegistration, ServerSetup,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[allow(dead_code)]
struct DefaultCipherSuite;
impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = opaque_ke::ksf::Identity;
}

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
pub async fn register(
    form: web::Form<RegisterForm>,
    templates: web::Data<tera::Tera>,
) -> impl Responder {
    let username = form.email.clone();
    let password = form.password.clone();

    println!("Email: {}, Password: {}", form.email, form.password);
    let mut client_rng = OsRng;

    // This is only for registration
    // Start client registration
    // Client function
    let client_registration_start_result =
        ClientRegistration::<DefaultCipherSuite>::start(&mut client_rng, password.as_bytes())
            .unwrap();
    let registration_request_bytes = client_registration_start_result.message.serialize();

    // Server function
    // The server needs setup
    let mut rng = OsRng;
    let server_setup = ServerSetup::<DefaultCipherSuite>::new(&mut rng);

    let server_registration_start_result = ServerRegistration::<DefaultCipherSuite>::start(
        &server_setup,
        RegistrationRequest::deserialize(&registration_request_bytes).unwrap(),
        username.as_bytes(),
    )
    .unwrap();
    let registration_response_bytes = server_registration_start_result.message.serialize();

    // Client function
    let client_finish_registration_result = client_registration_start_result
        .state
        .finish(
            &mut client_rng,
            password.as_bytes(),
            RegistrationResponse::deserialize(&registration_response_bytes).unwrap(),
            ClientRegistrationFinishParameters::default(),
        )
        .unwrap();
    let message_bytes = client_finish_registration_result.message.serialize();

    // Client sends message_bytes to server
    let password_file = ServerRegistration::finish(
        RegistrationUpload::<DefaultCipherSuite>::deserialize(&message_bytes).unwrap(),
    );
    let ser = password_file.serialize(); // This is the file to store
    let vec: Vec<u8> = ser.to_vec();
    println!("Serialized password file: {:?}", vec);
    let base64_encoded_vec = encode(&vec);

    // Prepare the context for rendering the success page
    let mut context = tera::Context::new();
    context.insert("email", &form.email);
    context.insert("password_file", &base64_encoded_vec);

    // Render the success template after registration
    match templates.render("login-success.html", &context) {
        Ok(s) => HttpResponse::Ok().content_type("text/html").body(s),
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError()
                .content_type("text/html")
                .body("<p>Something went wrong!</p>")
        }
    }
}
