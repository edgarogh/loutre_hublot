#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;

use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{FileTransport, Message, SmtpTransport, Transport};
use rocket::http::hyper::header::Location;
use rocket::request::Form;
use rocket::State;
use std::time::Instant;

struct Mailer {
    pub from: Mailbox,
    pub to: Mailbox,

    pub transport: SmtpTransport,
    pub transport_fallback: FileTransport,

    pub error_message: &'static str,
    pub redirect_to: &'static str,
}

#[derive(Responder)]
#[response(status = 303)]
struct RawRedirect((), Location);

impl RawRedirect {
    fn to(uri: impl Into<String>) -> Self {
        RawRedirect((), Location(uri.into()))
    }
}

#[derive(FromForm)]
struct ContactForm {
    #[form(field = "first-name")]
    first_name: String,
    #[form(field = "last-name")]
    last_name: String,
    email: String,
    subject: String,
    message: String,
}

#[post("/contact", data = "<form>")]
fn contact(form: Form<ContactForm>, mailer: State<Mailer>) -> Result<RawRedirect, &'static str> {
    let ContactForm {
        first_name,
        last_name,
        email,
        subject,
        message,
    } = form.into_inner();

    let email = Message::builder()
        .from(mailer.from.clone())
        .to(mailer.to.clone())
        .subject(format!(
            "{} {} <{}> – {}",
            first_name, last_name, email, subject
        ))
        .body(message)
        .unwrap();

    let time = Instant::now();
    match mailer.transport.send(&email) {
        Ok(_) => {
            info!("e-mail took {:?} to send", time.elapsed());
            Ok(RawRedirect::to(mailer.redirect_to))
        }
        Err(err) => {
            error!("couldn't send e-mail: {:?}", err);
            error!(
                "  attempting to save e-mail as file: {:?}",
                mailer.transport_fallback.send(&email),
            );
            Err(mailer.error_message)
        }
    }
}

fn main() {
    let _ = dotenv::dotenv();

    let from = std::env::var("LH_FROM").unwrap();
    let from_addr = from.parse().unwrap();

    let credentials = Credentials::new(from, std::env::var("LH_PASSWORD").unwrap());

    let transport = SmtpTransport::relay(&std::env::var("LH_SERVER").unwrap())
        .unwrap()
        .authentication(vec![Mechanism::Plain])
        .credentials(credentials)
        .build();

    let error_message = match std::env::var("LH_ERROR_MESSAGE") {
        Ok(error_message) => Box::leak(error_message.into_boxed_str()),
        Err(_) => "An error occurred while sending the form.",
    };

    let redirect_to = match std::env::var("LH_REDIRECT") {
        Ok(redirect) => Box::leak(redirect.into_boxed_str()),
        Err(_) => "/",
    };

    let mailer = Mailer {
        from: Mailbox::new(Some("LoutreHublot".into()), from_addr),
        to: Mailbox::new(None, std::env::var("LH_TO").unwrap().parse().unwrap()),

        transport,
        transport_fallback: FileTransport::new("."),

        error_message,
        redirect_to,
    };

    rocket::ignite()
        .manage(mailer)
        .mount("/", routes![contact])
        .launch();
}
