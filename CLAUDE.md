# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Berserk is a Rust graphics application built with wgpu and winit. It renders a simple triangle using modern GPU APIs through the wgpu abstraction layer. The project uses Rust edition 2024.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Build for release
cargo build --release

# Run with logging enabled (shows GPU/wgpu debug info)
RUST_LOG=debug cargo run

# Check code without building
cargo check

# Run clippy linter
cargo clippy

# Format code
cargo fmt
```

**IMPORTANT**: When running `cargo run`, always use the Bash tool's `run_in_background` parameter. This is a GUI application that opens a window and runs continuously until closed by the user. If you need to restart the app, kill the previous shell first using KillShell.

## Architecture

The application follows a modular event-driven architecture split across three main modules:

### Core Components

- **main.rs**: Entry point that initializes env_logger and creates the winit event loop and App
- **app.rs**: Implements winit's ApplicationHandler trait, managing the application lifecycle and window events. Stores a leaked 'static reference to the Window to satisfy GPU surface lifetime requirements
- **gpu.rs**: Encapsulates all wgpu state (surface, device, queue, pipeline, configuration). Handles GPU initialization, window resizing, and frame rendering

### Lifetime Management

The codebase uses a deliberate pattern of leaking resources to obtain 'static lifetimes:
- Window is leaked in app.rs:34 to create a &'static Window reference
- wgpu::Instance is leaked in gpu.rs:21 to create a Surface<'static>

This approach simplifies lifetime management for this demo application but should be reconsidered for production code.

### GPU Initialization Flow

1. Event loop creates window on resume (app.rs:26-42)
2. GpuState::new() is called synchronously using pollster::block_on
3. GPU initialization creates: Instance → Surface → Adapter → Device/Queue → Surface Config → Shader → Pipeline
4. Shader source is loaded from shaders/triangle.wgsl at compile time via include_str!

### Rendering

- Continuous redraw model: window.request_redraw() is called after every frame (app.rs:75)
- Render pipeline draws a single hardcoded triangle (3 vertices, no vertex buffer)
- Vertex positions are defined in the WGSL shader using builtin(vertex_index)
- Surface errors (Outdated, Lost) trigger automatic reconfiguration

## Shaders

WGSL shaders are located in `shaders/`:
- triangle.wgsl: Vertex and fragment shaders for the basic triangle

Shaders are embedded at compile time, so changes require a rebuild.

## Dependencies

Key dependencies:
- **wgpu 27**: Cross-platform GPU API abstraction
- **winit 0.30**: Window creation and event handling
- **pollster 0.4**: Blocks on async functions (used for GPU initialization)
- **log + env_logger**: Logging infrastructure (use RUST_LOG env var to control output)
