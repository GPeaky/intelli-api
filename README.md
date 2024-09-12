<p align="center">
    <img src="https://avatars.githubusercontent.com/u/158355068?s=400&u=dd74b7a8edf3863336bf4cbc03a26c1c450f424f&v=4" style="width:20vh;" >
</p>
<h1 align="center">Intelli Telemetry</h1>

## Table of Contents
- [About](#about)
- [Built With](#built-with)
- [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
    - [Installation](#installation)
- [Usage](#usage)
- [Performance Optimizations](#performance-optimizations)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Authors](#authors)

## About
Intelli Telemetry is a high-performance, real-time data collection and management system for F1 leagues. It provides comprehensive race session data, championship management, and analytics for teams, spectators, and league organizers, with a focus on maximum efficiency and minimal latency.

## Built With
The project is built using cutting-edge technologies and libraries optimized for performance:

- [Rust](https://www.rust-lang.org) (nightly) - Systems programming language known for safety and performance
- [Tokio](https://tokio.rs/) - Asynchronous runtime for Rust
- [Ntex](https://ntex.rs/) - Actor framework for Rust
- [PostgreSQL](https://www.postgresql.org/) - Advanced open-source relational database

## Getting Started

### Prerequisites
Before you begin, ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install) (nightly version)
- [PostgreSQL](https://www.postgresql.org/download/) (13.0 or higher)
- An email service account (for notifications)

### Installation
1. Install Rust nightly:
   ```sh
   rustup default nightly
   ```

2. Clone the repository:
   ```sh
   git clone https://github.com/Intelli-Telemetry/api.git
   cd api
   ```

3. Set up your PostgreSQL database.

4. Create a `.env` file in the project root with the following content:
   ```env
   HOST=0.0.0.0
   DATABASE_URL=postgres://username:password@localhost/database_name
   EMAIL_HOST=smtp.your-email-provider.com
   EMAIL_FROM=your-email@example.com
   EMAIL_NAME=Your Name
   EMAIL_PASS=your-email-password
   DISCORD_CLIENT_ID=your-discord-client-id
   DISCORD_CLIENT_SECRET=your-discord-client-secret
   DISCORD_REDIRECT_URI=http://localhost:3000/auth/discord/callback
   ```
   Replace the placeholders with your actual credentials and settings.

5. Build and run the project:
   ```sh
   cargo run --release
   ```

## Usage
Intelli Telemetry provides a comprehensive solution for F1 leagues, offering:

- Real-time race session data with microsecond precision
- Live timing and telemetry with minimal latency
- Automated championship standings
- Penalty and incident reporting system
- Detailed analytics for teams and drivers

For detailed API documentation, visit our [GitBook](https://gerardjoven2020.gitbook.io/intelli-api/).

## Performance Optimizations
Intelli Telemetry is designed for maximum performance:

- Zero-copy deserialization for incoming data streams
- Lock-free concurrent data structures for high-throughput scenarios
- Custom memory allocator (mimalloc) for improved memory management
- Aggressive inlining and SIMD optimizations where applicable
- Careful use of `unsafe` code in performance-critical paths, thoroughly tested and documented

To enable all optimizations, build with:
```sh
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Roadmap
See the [open issues](https://github.com/GPeaky/intelli-api/issues) for a list of proposed features and known issues.

## Contributing
Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

Please read through our [Code of Conduct](https://github.com/GPeaky/intelli-api/blob/main/CODE_OF_CONDUCT.md) before contributing.

## License
Distributed under the MIT License. See [LICENSE](https://github.com/GPeaky/intelli-api/blob/main/LICENSE.md) for more information.

## Authors
- **Gerard Zinn** - [GitHub Profile](https://github.com/GPeaky)