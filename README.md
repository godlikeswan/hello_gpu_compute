# hello_gpu_compute

A small collection of simple, progressive Rust examples demonstrating GPU compute. Started with [wgpu](https://github.com/gfx-rs/wgpu), but designed to be flexible as more backends/examples are added.

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- A GPU with Vulkan/Metal/DirectX 12 support (wgpu will pick a backend automatically)

## Running Examples

Examples are in the `examples/` directory and can be run with Cargo:

```bash
cargo run --example 01_cpu_vs_gpu
```

## Examples

| # | Name | Description |
|---|------|-------------|
| 01 | [cpu_vs_gpu](examples/01_cpu_vs_gpu) | Compares CPU and GPU computation of a 300-iteration xorshift hash over 16M integers. Verifies identical results and prints timing for both. |

## Notes

- The GPU timings include setup costs (adapter/device/buffer creation), not just kernel execution time.
- Each example is self-contained with its own shader when needed.

## License

[MIT](LICENSE).
