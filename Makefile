.PHONY: setup build

setup: app_init update apply

app_init:
	cd app && pnpm i

update:
	git submodule update --init --recursive

apply:
	cd chat && git add . && git diff --cached > ../patches/jChat.patch

patch:
	cd chat && git apply ../patches/jChat.patch

unpatch:
	cd chat && git reset --hard

commit:
	make patch
	git commit -S -a

pool:
	cargo r -p twitch_api

check:
	cargo check
	cargo clippy

build: pool
	cd app && pnpm tauri build

run: pool
	cd app && pnpm tauri dev
