mod network;
mod world;

use net::{Server, ServerError};
use network::Driver;
use tokio::{join, signal, task::JoinError};

#[derive(Debug)]
enum Error {
    ServerError(ServerError),
    JoinError(JoinError),
}

impl From<ServerError> for Error {
    fn from(err: ServerError) -> Self {
        Self::ServerError(err)
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Self::JoinError(err)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let server = Server::start("0.0.0.0", 19980, Driver::new());

    signal::ctrl_c().await.ok();

    join!(server.stop()).0.ok();

    // let (terminate, mut handle) = network::server::start("0.0.0.0", 19980);

    // tokio::select! {
    //     _ = signal::ctrl_c() => {
    //         server.stop()
    //         terminate.send(()).ok();
    //         handle.await??
    //     }
    //     result = &mut handle => {
    //         result??
    //     }
    // }

    Ok(())

    // join!(network::server::start_new(
    //     // std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
    //     // std::sync::Arc::new(std::sync::Mutex::new(world::world::World::new())),
    //     // std::sync::Arc::new(std::sync::Mutex::new(network::handler::Handler::new())),
    //     "0.0.0.0", 19980,
    // ))
    // .0
    // .map_err(|err| format!("server start failure due to: {}", err))?;

    // Ok(())
}
