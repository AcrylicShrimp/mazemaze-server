mod network;

fn main() {
    network::server::Server::new().start("0.0.0.0", 19980);
}
