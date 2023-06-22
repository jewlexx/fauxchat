use std::io::Read;

include!("src/creds/decl.rs");

impl Credentials {
    fn load_from_env() -> Result<Self, std::env::VarError> {
        use std::env::var;

        Ok(Self {
            client_id: var("TWITCH_CLIENT_ID")?,
            client_secret: var("TWITCH_CLIENT_SECRET")?,
            user_id: var("TWITCH_USER_ID")?,
            refresh_token: var("TWITCH_REFRESH_TOKEN")?,
            auth_token: var("TWITCH_AUTH_TOKEN")?,
        })
    }
}

fn main() {
    use std::fs::File;

    let pwd = std::env::current_dir().unwrap();

    let creds: Credentials = {
        let creds_path = pwd.join("../../credentials.toml");

        if !creds_path.exists() {
            use std::io::Write;

            let creds = Credentials::load_from_env()
                .expect("valid credentials provided in env, or credentials file");

            let creds_string = toml::to_string_pretty(&creds).unwrap();

            File::create(creds_path)
                .unwrap()
                .write_all(creds_string.as_bytes())
                .unwrap();

            creds
        } else {
            println!("cargo:rerun-if-changed={}", creds_path.display());

            let mut creds_file = File::open(creds_path).unwrap();

            let mut creds = String::new();

            creds_file.read_to_string(&mut creds).unwrap();

            toml::from_str(&creds).unwrap()
        }
    };

    // Make credentionals accessible within the program
    println!("cargo:rustc-env=TWITCH_CLIENT_ID={}", creds.client_id);
    println!(
        "cargo:rustc-env=TWITCH_CLIENT_SECRET={}",
        creds.client_secret
    );
    println!("cargo:rustc-env=TWITCH_USER_ID={}", creds.user_id);
    println!("cargo:rustc-env=TWITCH_AUTH_TOKEN={}", creds.auth_token);
    println!(
        "cargo:rustc-env=TWITCH_REFRESH_TOKEN={}",
        creds.refresh_token
    );
}
