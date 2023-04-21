.PHONY: setup build

setup: update apply

update:
	git submodule update --init --recursive

apply:
	cd chat && git diff > ../patches/jChat.patch

patch:
	cd chat && git apply ../patches/jChat.patch

unpatch:
	cd chat && git reset --hard

commit:
	make patch
	git commit -S -a

build:
	cargo build --release