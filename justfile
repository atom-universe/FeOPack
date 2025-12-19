#!/usr/bin/env -S just --justfile

# Set shell configurations
set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

# Default target: List all tasks with updated information
_default:
    just --list -u

# Build all Rust crates and generate JS bindings for NAPI crates
build-crates:
    cargo build --workspace --release
    napi build --platform --release --package node-binding

# Build everything and update dependencies
build-update:
    just build-crates
    pnpm i
