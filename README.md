<p align="center">
    <img src="https://cdn.discordapp.com/attachments/1086116819644985345/1127742819591794729/31_sin_titulo_20230710012713.png" style="width:20vh;" >
</p>
<h1 align="center">Intelli</h1>

## Table of Contents

- [Build With](#build-with)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
- [RoadMap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Authors](#authors)

## Build With

The main Libraries/Frameworks/Language that i used to build this package

- [Rust](https://www.rust-lang.org)
- [Tokio](https://tokio.rs/)
- [Axum](https://crates.io/crates/axum)
- [Postgresql](https://www.postgresql.org/)
- [Redis](https://redis.io/)

## Getting Started

### Prerequisites

- Rust
- Email Service
- Postgresql Database
- Redis Database

```sh
    cargo run
```

### Installation

Installation Command

```sh
    git clone https://github.com/GPeaky/intelli-api.git
```

You need a .env to run the project. This is an example of what that .env should have

```env
    # Application
    HOST=0.0.0.0:3000
    REDIS_URL=
    PASS_SALT=""
    JWT_SECRET=""

    # Scylla Database
    SCYLLA_URI=
    SCYLLA_USER=
    SCYLLA_PASS=
    SCYLLA_KEYSPACE=

    # Email
    EMAIL_HOST=
    EMAIL_NAME=
    EMAIL_PASS=
```

## Usage

This project is created to provide a new solution to f1 leagues, to have real time data about their races sessions, Championship Manager, and everything about an F1 League. At the same time we want to give all for free

[Documentation](https://gerardjoven2020.gitbook.io/intelli-api/)

## RoadMap

See the [open issues](https://github.com/GPeaky/intelli-api/issues) for list of proposed features and fix errors (and known issues).

## Contributing

Contributions are what make the open source community such an amazing place to be learn, inspire, and create. Any contributions you make are **greatly appreciated**.

- If you have suggestions for adding or removing projects, feel free to [open an issue](https://github.com/GPeaky/intelli-api/issues/new) to discuss it, or directly create a pull request after you edit the _README.md_ file with necessary changes.
- Please make sure you check your spelling and grammar.
- Create individual PR for each suggestion.
- Please also read through the [Code Of Conduct](https://github.com/GPeaky/intelli-api/blob/main/CODE_OF_CONDUCT.md) before posting your first idea as well.

### Creating A Pull Request

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

Distributed under the MIT License. See [LICENSE](https://github.com/GPeaky/intelli-api/blob/main/LICENSE.md) for more information.

## Authors

- **Gerard Zinn** - **[Gerard Zinn](https://github.com/GPeaky)**
