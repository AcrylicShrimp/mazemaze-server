use bytes::{Bytes, BytesMut};
use parking_lot::Mutex;
use std::io::{Error, ErrorKind, Result};
use std::ptr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::{select, spawn};

pub type ClientId = *const Mutex<Client>;

pub struct Client {
    id: ClientId,
    join_handles: (JoinHandle<()>, JoinHandle<()>),
}

impl Client {
    pub fn from_stream(stream: TcpStream) -> Arc<Mutex<Client>> {
        let (reader, writer) = stream.into_split();
        let (read_in_sigterm_sender, read_in_sigterm_receiver) = channel();
        let (read_out_sigterm_sender, read_out_sigterm_receiver) = channel();
        let (write_in_sigterm_sender, write_in_sigterm_receiver) = channel();
        let (write_out_sigterm_sender, write_out_sigterm_receiver) = channel();
        let (in_packet_sender, in_packet_receiver) = unbounded_channel();
        let (out_packet_sender, out_packet_receiver) = unbounded_channel();
        let join_handles = (
            spawn(async move {
                read_task(
                    reader,
                    read_in_sigterm_receiver,
                    read_out_sigterm_sender,
                    in_packet_sender,
                )
                .await;
            }),
            spawn(async move {
                write_task(
                    writer,
                    write_in_sigterm_receiver,
                    write_out_sigterm_sender,
                    out_packet_receiver,
                )
                .await;
            }),
        );

        let client = Arc::new(Mutex::new(Client {
            id: ptr::null(),
            join_handles,
        }));

        {
            let mut locked_client = client.lock();
            locked_client.id = Arc::as_ptr(&client);
        }

        client
    }

    pub fn id(&self) -> ClientId {
        self.id
    }
}

async fn read_task(
    reader: OwnedReadHalf,
    mut sigterm_receiver: Receiver<()>,
    mut sigterm_sender: Sender<Error>,
    mut tx: UnboundedSender<Bytes>,
) {
    let mut reader = BufReader::with_capacity(128, reader);

    loop {
        let size = {
            select! {
                _ = &mut sigterm_receiver => {
                    break;
                }
                size = reader.read_u16_le() => match size {
                    Ok(size) => size,
                    Err(err) => {
                        match sigterm_sender.send(err) {
                            _ => {}
                        }
                        return;
                    }
                }
            }
        };

        if 2048 < size {
            match sigterm_sender.send(Error::new(
                ErrorKind::InvalidData,
                "packet size exceeeds 2048 bytes",
            )) {
                _ => {}
            }
            return;
        }

        let mut buffer = BytesMut::with_capacity(size as usize);
        let buffer = select! {
            _ = &mut sigterm_receiver => {
                break;
            }
            result = reader.read_exact(&mut buffer) => {
                match result {
                    Ok(..) => {},
                    Err(err) => {
                        match sigterm_sender.send(err) {
                            _ => {}
                        }
                        return;
                    }
                }
                buffer.freeze()
            }
        };

        match tx.send(buffer) {
            Ok(..) => {}
            Err(err) => {
                match sigterm_sender.send(Error::new(ErrorKind::BrokenPipe, err)) {
                    _ => {}
                }
                return;
            }
        }
    }
}

async fn write_task(
    mut writer: OwnedWriteHalf,
    mut sigterm_receiver: Receiver<()>,
    mut sigterm_sender: Sender<Error>,
    mut rx: UnboundedReceiver<Bytes>,
) -> Result<()> {
    Ok(())
}
