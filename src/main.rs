use tokio::join;

mod network;
mod world;

#[tokio::main]
async fn main() -> Result<(), String> {
    join!(network::server::start_new(
        // std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        // std::sync::Arc::new(std::sync::Mutex::new(world::world::World::new())),
        // std::sync::Arc::new(std::sync::Mutex::new(network::handler::Handler::new())),
        "0.0.0.0", 19980,
    ))
    .0
    .map_err(|err| format!("server start failure due to: {}", err))?;

    Ok(())
}
