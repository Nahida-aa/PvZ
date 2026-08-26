run-debug:
    cargo run

dev:
    cargo watch -x run

run-release:
    cargo run --release

check:
    cargo check

compose-lawnmower:
    cargo run -p plant_composer -- --input assets/items/lawnmower --output tmp/lawnmower_composed.png

compose-imitater:
    cargo run -p plant_composer -- --input assets/plants/imitater --output tmp/imitater_composed.png
