# Lumilly Render

Monte Carlo path tracing implementation on Rust

![image](sample.png)

1920x1080 10000 samples per pixel

## Run

```
RUSTFLAGS='--emit asm -C target-feature=+avx' cargo run --release
```
