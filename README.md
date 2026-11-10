# AK4497DAC

Raspberry Pi Pico (RP2040) + embassy-rp による AK4497 制御ファームウェア。

- ピン: GP4=I2C0 SDA, GP5=I2C0 SCL, GP6=PDN, I2Cアドレス 0x10 (CADピンで変更)
- ビルド: `rustup target add thumbv6m-none-eabi && cargo build --release`
- 書き込み: `cargo run --release` (probe-rs)
- `src/ak4497.rs`: レジスタ制御 (フォーマット, デエンファシス, 音量, ミュート)
