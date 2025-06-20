
# Simple Imageboard in Rust

see demo.png for a screenshot 

A minimal web imageboard built with Rust and Actix-Web to demonstrate how quickly you can create a fully functional app—even with no database configuration. Perfect as a learning exercise to show that Rust development can be both powerful and approachable.

## Features

* **Image & Video Posts**: Supports uploads of PNG, JPG, GIF, WEBP, and MP4 files.
* **Persistent Storage with Sled**: Embedded, zero-configuration key-value store—no external database server required.
* **Modern UI**: Inline CSS for a clean, responsive look.
* **Self-Contained**: All code and assets live locally; cloning and running is all you need.

## Getting Started

### Prerequisites

* Rust (1.60 or later)
* Cargo (bundled with Rust)

### Installation & Run

```bash
# Clone the repository
git clone <your-repo-url>
cd simple_imageboard

# Build and start the server
cargo run
```

Open your browser to `http://localhost:8080` and start posting!

## Why Sled?

[Sled](https://sled.rs/) is an embedded key-value store written in Rust:

* **Zero Configuration**: Just open or create the database directory—no setup or tuning.
* **Pure Rust**: No external dependencies or servers to manage.
* **High Performance**: Optimized for modern hardware.

In this demo, each post is serialized into JSON and stored under an auto-generated ID. The app iterates entries in reverse to display the newest posts first.

## Why This Demo Exists

This simple imageboard shows that:

1. **Rust is approachable**: You can build a web app in under 200 lines of code.
2. **Ecosystem strength**: Mature crates like Actix-Web and Sled make development fast.
3. **AI-powered scaffolding**: Tools like AI assistants can generate working Rust code in minutes—making learning and prototyping even easier.

Feel free to fork, customize, and extend this demo to continue your Rust journey!
