# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
This is a Rust learning project to create a gRPC client for SPIRE Server APIs. The implementation will be done by the user, and Claude Code's role is to provide tutorials, guidance, and advice.

## Development Approach
- Provide educational guidance and explanations about Rust concepts
- Offer architectural advice for gRPC client implementation
- Suggest best practices for Rust development
- Help troubleshoot issues and explain error messages
- Do NOT implement code directly unless specifically asked

## Common Commands
```bash
# Create new Rust project
cargo init

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Key Technologies
- Rust programming language
- gRPC for API communication
- SPIRE Server APIs
- mTLS for secure communication

## Learning Resources to Suggest
- The Rust Programming Language book
- Rust async programming concepts
- tonic crate for gRPC in Rust
- tokio runtime for async Rust
- SPIRE Server API documentation

## Development Guidelines
- コードの実装・修正はクレートのバージョンに対応した方法か確認すること
