# Contraband

Contraband is a web framework built on top of actix for creating modular
applications in Rust with dependency injection and performant higher-level
abstractions.

Contraband is heavily inspired by Spring Boot and Nestjs.

For more information you can check out the
[examples](https://github.com/styren/contraband/tree/master/examples).

## First steps

Create an empty project with Cargo
```bash
$ cargo new example-project
$ cd example-project
```

Add contraband and actix\_web to the list of dependencies in `Cargo.toml`
```toml
[dependencies]
actix-web = "3"
contraband = "^0.0.1"
contraband_codegen = "^0.0.1"
```

### Controllers

Controllers are responsible for handling incoming request and is able to serve
multiple routes and HTTP-method types.

In order to create a controller we use **structs** and **attributes**. Attributes
are used to generate the boilerplate necessary to register all routes and
inject all dependencies into the controller.

```rust
use contraband::{Injectable, controller};
use contraband::core::ContrabandApp;
use actix_web::HttpResponse;

#[derive(Clone, Injectable)]
struct HelloController;

#[controller]
impl HelloController {
    #[get]
    async fn hello_world(self) -> HttpResponse {
        HttpResponse::Ok().body("Hello world!")
    }
}
```

Note the derive clause. In order to be injected into the Contraband app it
needs to derive **Injectable**. Since Contraband spawns multiple instances it
also needs to derive **Clone**.

### Providers

One of the biggest perks of Contraband is the ability to inject arbitrary
dependencies into, for example, our controllers. This is done through deriving
the aforementioned **Injectable**-trait.

Below is and example of how an injectable struct can be used for user management.

```rust
#[derive(Clone, Injectable)]
pub struct UserService;

impl UserService {
    pub async fn add_user(&self, name: &str) {
        // stub
    }

    pub async fn get_user(&self, id: u32) {
        // stub
    }
}
```

In order to utilize our new service we can inject it into our `HelloController`
by defining an `Arc`-pointer to our service.

```rust
// ...
#[derive(Clone, Injectable)]
struct HelloController {
    user_service: std::sync::Arc<UserService>
}
// ...
```

We are now able to use our new service in any methods in `HelloController`.
Below we use our new service for fetching users in a new route.

```rust
#[controller]
// ...
impl HelloController {
    // ...
    #[get("/users/:id")]
    async fn get_users(self, id: actix_web::web::Path<i32>) -> HttpResponse {
        let name = self.user_service.get_user(id);
        HttpResponse::Ok().body(name)
    }
}
// ...
```

However in order for the dependency to be resolved when initializing the
Contraband runtime we need to register it as a provider. More on that in the
next chapter.

### Modules

A module is a struct that derives the **module**-trait and is used to organize
our controllers and dependencies into nested building blocks.

```rust
use contraband::core::ContrabandApp;
use contraband::module;

#[module]
#[controller(HelloController)]
#[provider(UserService)]
struct AppModule;

#[contraband::main]
async fn main() -> std::io::Result<()> {
    ContrabandApp::new()
        .start::<AppModule>()
        .await
}
```

Note how we registered our **UserService** as a provider. This is required in
order to inject the dependency in this scope and enabling its use in our
controller.

## License

Contraband is licensed under either [MIT licensed](LICENSE-MIT) or
[Apache 2.0 licensed](LICENSE-APACHE).
