import tomli_w
from os import environ

with open("credentials.toml", "x") as f:
    credentials = {
        "client_id": environ["CLIENT_ID"],
        "client_secret": environ["CLIENT_SECRET"],
        "user_id": environ["USER_ID"],
        "auth_token": environ["AUTH_TOKEN"],
        "refresh_token": environ["REFRESH_TOKEN"],
    }

    tomli_w.dump(credentials, f)
