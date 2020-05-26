mod network;
mod world;

fn main() {
    network::server::start(
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        std::sync::Arc::new(std::sync::Mutex::new(world::world::World::new())),
        std::sync::Arc::new(std::sync::Mutex::new(network::handler::Handler::new())),
        "127.0.0.1",
        19980,
    );
}
