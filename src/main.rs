#[macro_use]
extern crate log;

use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{AsyncTransport, Message};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::{post, routes, FromForm, State};
use std::time::Instant;

type AsyncSmtpTransport = lettre::AsyncSmtpTransport<lettre::Tokio1Executor>;
type AsyncFileTransport = lettre::AsyncFileTransport<lettre::Tokio1Executor>;

struct Mailer {
    pub from: Mailbox,
    pub to: Mailbox,

    pub transport: AsyncSmtpTransport,
    pub transport_fallback: AsyncFileTransport,

    pub error_message: &'static str,
    pub redirect_to: &'static str,
}

#[derive(FromForm)]
struct ContactForm {
    #[field(name = "first-name")]
    first_name: String,
    #[field(name = "last-name")]
    last_name: String,
    email: String,
    subject: String,
    message: String,
}

#[post("/contact", data = "<form>")]
async fn contact(
    form: Form<ContactForm>,
    mailer: &State<Mailer>,
) -> Result<Redirect, &'static str> {
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
        .subject(format!("{first_name} {last_name} <{email}> – {subject}"))
        .header(ContentType::TEXT_PLAIN)
        .body(message)
        .unwrap();

    let time = Instant::now();
    match mailer.transport.send(email.clone()).await {
        Ok(_) => {
            info!("e-mail took {:?} to send", time.elapsed());
            Ok(Redirect::to(mailer.redirect_to))
        }
        Err(err) => {
            error!("couldn't send e-mail: {:?}", err);
            error!(
                "  attempting to save e-mail as file: {:?}",
                mailer.transport_fallback.send(email).await,
            );
            Err(mailer.error_message)
        }
    }
}

// The `async` is required for #[rocket::launch] to run this function inside a tokio reactor
#[allow(clippy::unused_async)]
#[rocket::launch]
async fn launch() -> _ {
    let _ = dotenv::dotenv();

    let from = std::env::var("LH_FROM").unwrap();
    let from_addr = from.parse().unwrap();

    let credentials = Credentials::new(
        std::env::var("LH_USER").unwrap_or(from),
        std::env::var("LH_PASSWORD").unwrap(),
    );

    let transport = AsyncSmtpTransport::relay(&std::env::var("LH_SERVER").unwrap())
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
        transport_fallback: AsyncFileTransport::new("."),

        error_message,
        redirect_to,
    };

    rocket::build().manage(mailer).mount("/", routes![contact])
}
