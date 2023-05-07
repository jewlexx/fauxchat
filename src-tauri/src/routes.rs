use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};

// User follows reference: https://dev.twitch.tv/docs/api/reference#get-users-follows
// And to get user id in the first place: https://dev.twitch.tv/docs/api/reference#get-users

#[actix_web::get("/twitch/{filename:.*}")]
async fn twitch(req: HttpRequest) -> Result<NamedFile> {
    let base_path = std::env::current_dir().expect("current working directory");
    let path = {
        let query = req.match_info().query("filename");

        if query.is_empty() {
            "index.html"
        } else {
            query
        }
    };

    let qualified_path = base_path.join("../chat").join(path);

    Ok(NamedFile::open_async(qualified_path).await?)
}

#[allow(clippy::unused_async)]
#[actix_web::get("/credentials.js")]
async fn credentials() -> Result<String> {
    let creds = faker::twitch_api::creds::Credentials::read();

    let client_id = creds.client_id;
    let api_token = creds.auth_token;

    let file = format!(
        r#"
const client_id = "{client_id}";
const credentials = "{api_token}";
        "#
    );

    Ok(file)
}
