use crate::{ClientManager, ServerDriver};
use std::io::Error as IOError;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{
    unbounded_channel as mpsc_channel, UnboundedReceiver as MpscReceiver,
    UnboundedSender as MpscSender,
};
use tokio::sync::oneshot::{
    channel as oneshot_channel, Receiver as OneshotReceiver, Sender as OneshotSender,
};
use tokio::task::JoinHandle;

pub struct Server {
    exit_signal_sender: OneshotSender<()>,
    handle: JoinHandle<Result<(), ServerError>>,
}

#[derive(Debug)]
pub enum ServerError {
    IOError(IOError),
}

impl From<IOError> for ServerError {
    fn from(err: IOError) -> Self {
        Self::IOError(err)
    }
}

impl Server {
    pub fn start<S: Into<String>, D: ServerDriver>(host: S, port: u16, driver: D) -> Self {
        let host = host.into();
        let driver = Arc::new(driver);
        let client_manager = Arc::new(ClientManager::new());

        let (exit_signal_sender, exit_signal_receiver) = oneshot_channel();
        let handle = tokio::spawn(async move {
            let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;
            run_task(driver, listener, client_manager, exit_signal_receiver).await;
            Ok(())
        });

        Self {
            exit_signal_sender,
            handle,
        }
    }

    pub fn stop(self) -> JoinHandle<Result<(), ServerError>> {
        self.exit_signal_sender.send(()).ok();
        self.handle
    }
}

async fn run_task(
    driver: Arc<dyn ServerDriver>,
    listener: TcpListener,
    client_manager: Arc<ClientManager>,
    exit_signal_receiver: OneshotReceiver<()>,
) {
    let (termination_signal_sender, termination_signal_receiver) = mpsc_channel();

    let (killer_exit_signal_sender, killer_exit_signal_receiver) = oneshot_channel();
    let (listener_exit_signal_sender, listener_exit_signal_receiver) = oneshot_channel();

    let killer_handle = tokio::spawn(killer_task(
        driver.clone(),
        client_manager.clone(),
        killer_exit_signal_receiver,
        termination_signal_receiver,
    ));
    let listener_handle = tokio::spawn(listener_task(
        driver,
        listener,
        client_manager.clone(),
        termination_signal_sender,
        listener_exit_signal_receiver,
    ));

    exit_signal_receiver.await.ok();

    listener_exit_signal_sender.send(()).ok();
    tokio::join!(listener_handle).0.ok();

    // TODO: Send a server closed packet to all clients.
    client_manager.for_each_client(|client| {
        match client.io_pump_exit_signal_sender().deref_mut().take() {
            Some(sender) => {
                sender.send(()).ok();
            }
            None => {}
        }
    });

    killer_exit_signal_sender.send(()).ok();
    tokio::join!(killer_handle).0.ok();
}

async fn killer_task(
    driver: Arc<dyn ServerDriver>,
    client_manager: Arc<ClientManager>,
    mut exit_signal_receiver: OneshotReceiver<()>,
    mut termination_signal_receiver: MpscReceiver<usize>,
) {
    loop {
        tokio::select! {
            _ = &mut exit_signal_receiver => {
                break;
            }
            id = termination_signal_receiver.recv() => match id {
                Some(id) => if client_manager.remove_client(id) {
                    driver.on_client_out(id).await;
                }
                None => break,
            }
        }
    }
}

async fn listener_task(
    driver: Arc<dyn ServerDriver>,
    mut listener: TcpListener,
    client_manager: Arc<ClientManager>,
    termination_signal_sender: MpscSender<usize>,
    mut exit_signal_receiver: OneshotReceiver<()>,
) -> Result<(), ServerError> {
    loop {
        tokio::select! {
            _ = &mut exit_signal_receiver => {
                break;
            }
            result = listener.accept() => {
                let (stream, addr) = result?;
                stream.set_nodelay(true)?;
                let client = client_manager.add_client(stream, termination_signal_sender.clone());
                driver.on_client_in(client, addr).await;
            }
        }
    }

    Ok(())
}
