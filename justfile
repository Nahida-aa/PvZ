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

compose-pea-shooter:
    cargo run -p plant_composer -- --input assets/plants/pea_shooter --output tmp/pea_shooter_composed.png

compose-sun-flower:
    cargo run -p plant_composer -- --input assets/plants/sun_flower --output tmp/sun_flower_composed.png
