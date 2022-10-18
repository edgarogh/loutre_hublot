# **loutre_hublot**
_Contact form mailing API_

**loutre_hublot** is a small microservice that exposes a `/contact` endpoint where a form can point to. Sending a request to the endpoint will cause an e-mail to be sent to a configured e-mail mailbox. It is intended to be used as a backend for a contact form in a static website, as it doesn't require the referring page to include dynamic content.

## Usage

  * Compile **loutre_hublot**
  * Set the following environment variables (`.env`s are loaded):
    ```bash
    LH_SERVER=mail.example.com
    LH_FROM=loutre-hublot@example.com
    LH_USER=loutre-hublot@example.com
    LH_PASSWORD=hunter2
    
    LH_TO=myself@example.com
    
    # kinda optional
    LH_ERROR_MESSAGE="An error occurred, you can try to contact me at myself@example.com"
    LH_REDIRECT=https://example.com/#success
    ```
  * Start **loutre_hublot** as a daemon (using systemd, screen, tmux or anything you prefer over these)
      * You can create a [`Rocket.toml`](https://rocket.rs/v0.4/guide/configuration/#rockettoml) file to configure the port amongst other things
  * Configure your reverse proxy to forward `/contact*` to **loutre_hublot**s endpoint
  * In your contact form, use `/contact` as the action, and set the method to POST. Use the following field names:
      * `first-name`
      * `last-name`
      * `email`
      * `subject`
      * `message`

## Upcoming features (or not)

  * Simple built-in captcha
