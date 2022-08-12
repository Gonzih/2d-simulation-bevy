# FIXME
wasm-it:
	cargo build --release --target wasm32-unknown-unknown --no-default-features
	wasm-bindgen --out-dir wasm/target --target web target/wasm32-unknown-unknown/release/simulation-core.wasm

run:
	cargo $@

test:
	cargo $@

release:
	cargo build --release
	cp -f ./target/release/simulation-core .

run-dyn:
	cargo run --features bevy/dynamic

nix-run:
	nix-shell shell.nix --run "make run"

nix-run-dyn:
	nix-shell shell.nix --run "make run-dyn"


nix-release:
	nix-shell shell.nix --run "make release"

bevy-deps:
	sudo apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev

rust-setup:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	rustup default stable

wasm-setup:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli

dev-setup:
	cargo install cargo-watch

wasm/target:
	git clone git@github.com:natural20-studio/wasm-demo.git $@
