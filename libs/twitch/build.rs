use std::io::Read;

include!("src/creds/decl.rs");

fn main() {
    let pwd = std::env::current_dir().unwrap();

    let creds: Credentials = {
        let creds_path = {
            let path = pwd.join("../../credentials.toml");
            dunce::canonicalize(path).expect("valid credentials path")
        };

        println!("cargo:rerun-if-changed={}", creds_path.display());

        if !creds_path.exists() {
            panic!("credentials.toml file does not exist");
        }

        let mut creds_file = std::fs::File::open(creds_path).unwrap();

        let mut creds = String::new();

        creds_file.read_to_string(&mut creds).unwrap();

        toml::from_str(&creds).unwrap()
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