use std::path::PathBuf;

use actix_web::{HttpRequest, HttpResponse};

// TODO: Actual errors not just option returned

#[derive(Debug, thiserror::Error)]
pub enum RouteError {
    #[error("Could not find the given path")]
    NotFound(#[from] std::io::Error),
    #[allow(dead_code)]
    #[error("Could not find the given path, in included files")]
    NotIncluded,
}

#[cfg(debug_assertions)]
fn get_file(path: &str) -> Result<Vec<u8>, RouteError> {
    use std::{env, fs};

    let chat_dir = env::current_dir()?.join("..").join("chat");
    let path = chat_dir.join(path);
    let contents = fs::read(path)?;

    Ok(contents)
}

#[cfg(not(debug_assertions))]
fn get_file(path: &str) -> Result<Vec<u8>, RouteError> {
    match include_dir::include_dir!("chat").get_file(path) {
        Some(file) => Ok(file.contents().to_vec()),
        None => Err(RouteError::NotIncluded),
    }
}

// User follows reference: https://dev.twitch.tv/docs/api/reference#get-users-follows
// And to get user id in the first place: https://dev.twitch.tv/docs/api/reference#get-users

fn mime_type<'a>(path: String) -> &'a str {
    let path = PathBuf::from(path);
    let ext = path.extension().unwrap();

    match ext.to_string_lossy().to_string().as_str() {
        "" => "text/plain",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "png" => "image/png",
        "gif" => "image/gif",
        "jpg" | "jpeg" => "image/jpeg",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[allow(clippy::unused_async)]
#[actix_web::get("/twitch/{filename:.*}")]
async fn twitch(req: HttpRequest) -> HttpResponse {
    let path = {
        let query = req.match_info().query("filename");

        if query.is_empty() {
            "index.html"
        } else {
            query
        }
    };

    if let Ok(path_contents) = get_file(path) {
        let contents = {
            let mut contents: Vec<u8> = vec![];
            if path.contains("script.js") {
                let prefix = format!(
                    "// Injected by server
// Port determined at runtime, based off environment variables, or a preset default of 8080
// URL will remain 127.0.0.1 unless future developments change it
const URL = '127.0.0.1';
const PORT = '{}';
// End injected section\n",
                    crate::net::port()
                );

                contents.extend(prefix.as_bytes());
            }

            contents.extend(path_contents);

            contents
        };

        let mime = mime_type(path.to_string());

        HttpResponse::Ok().content_type(mime).body(contents)
    } else {
        let notfound_page = get_file("404.html").expect("404 page");
        HttpResponse::NotFound()
            .content_type("text/html")
            .body(notfound_page)
    }
}

#[allow(clippy::unused_async)]
#[actix_web::get("/credentials.js")]
async fn credentials() -> HttpResponse {
    let creds = twitch_api::creds::Credentials::read();

    let client_id = creds.client_id;
    let api_token = creds.auth_token;

    let file = format!(
        r#"
const client_id = "{client_id}";
const credentials = "{api_token}";
        "#
    );

    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(file)
}
