use actix_web::{get, post, web, HttpResponse, Responder};
use opaque_ke::{ciphersuite::CipherSuite, ksf::Ksf, Ristretto255};
use opaque_ke::{
    ClientRegistration, ClientRegistrationFinishParameters, CredentialFinalization,
    CredentialResponse, RegistrationRequest, RegistrationResponse, RegistrationUpload,
    ServerRegistration, ServerSetup,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

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
pub async fn register(form: web::Form<RegisterForm>) -> impl Responder {
    let username = form.email.clone();
    let password = form.password.clone();

    println!("Email: {}, Password: {}", form.email, form.password);
    let mut client_rng = OsRng;

    // Start client registration
    let client_registration_start_result =
        ClientRegistration::<DefaultCipherSuite>::start(&mut client_rng, password.as_bytes())
            .unwrap();
    let registration_request_bytes = client_registration_start_result.message.serialize();

    let mut rng = OsRng;
    let server_setup = ServerSetup::<DefaultCipherSuite>::new(&mut rng);
    let server_registration_start_result = ServerRegistration::<DefaultCipherSuite>::start(
        &server_setup,
        RegistrationRequest::deserialize(&registration_request_bytes).unwrap(),
        username.as_bytes(),
    )
    .unwrap();

    // Client sends registration_request_bytes to server

    let server_registration_start_result = ServerRegistration::<DefaultCipherSuite>::start(
        &server_setup,
        RegistrationRequest::deserialize(&registration_request_bytes).unwrap(),
        username.as_bytes(),
    )
    .unwrap();
    let registration_response_bytes = server_registration_start_result.message.serialize();

    // Server sends registration_response_bytes to client

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
    let ser = password_file.serialize();
    let vec: Vec<u8> = ser.to_vec();
    println!("Serialized password file: {:?}", vec);

    // Handle form processing (e.g., save to database, authentication, etc.)
    // Return response
    HttpResponse::Ok().body(format!(
        "Received email: {}, password: {}",
        form.email, form.password
    ))
}
