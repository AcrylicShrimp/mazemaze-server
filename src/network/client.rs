use super::RawPacket;
use bytes::{Bytes, BytesMut};
use parking_lot::{Mutex, MutexGuard};
use std::convert::TryInto;
use std::mem::size_of;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{
    error::SendError as MpscSendError, unbounded_channel as mpsc_channel,
    UnboundedReceiver as MpscReceiver, UnboundedSender as MpscSender,
};
use tokio::sync::oneshot::{
    channel as oneshot_channel, error::RecvError as OneshotRecvError, Receiver as OneshotReceiver,
    Sender as OneshotSender,
};
use tokio::{io::Error as IOError, task::JoinHandle};

pub struct Client {
    id: usize,
    tx_sender: Mutex<MpscSender<Bytes>>,
    rx_receiver: Mutex<Option<MpscReceiver<RawPacket>>>,
    io_pump_exit_signal_sender: Mutex<Option<OneshotSender<()>>>,
    io_pump_handle: JoinHandle<Result<(), IOPumpError>>,
}

impl Client {
    pub fn new(stream: TcpStream, termination_signal_sender: MpscSender<usize>) -> Arc<Self> {
        let (tx_sender, tx_receiver) = mpsc_channel::<Bytes>();
        let (rx_sender, rx_receiver) = mpsc_channel::<RawPacket>();
        let (io_pump_exit_signal_sender, io_pump_exit_signal_receiver) = oneshot_channel();

        let (client_id_sender, client_id_receiver) = oneshot_channel();
        let io_pump_handle = tokio::spawn(async move {
            let client_id = client_id_receiver.await?;
            let result = pump_stream(
                client_id,
                stream,
                tx_receiver,
                rx_sender,
                io_pump_exit_signal_receiver,
            )
            .await;
            termination_signal_sender.send(client_id).ok();
            return result;
        });

        let mut client = Arc::new(Client {
            id: 0,
            tx_sender: tx_sender.into(),
            rx_receiver: Some(rx_receiver).into(),
            io_pump_exit_signal_sender: Some(io_pump_exit_signal_sender).into(),
            io_pump_handle,
        });

        let client_id = Arc::as_ptr(&mut client) as _;
        let mut client_mut = Arc::get_mut(&mut client).unwrap();

        client_mut.id = client_id;
        client_id_sender.send(client_id).ok();

        client
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn tx_sender(&self) -> MutexGuard<MpscSender<Bytes>> {
        self.tx_sender.lock()
    }

    pub fn tx_receiver(&self) -> MutexGuard<Option<MpscReceiver<RawPacket>>> {
        self.rx_receiver.lock()
    }

    pub fn io_pump_exit_signal_sender(&self) -> MutexGuard<Option<OneshotSender<()>>> {
        self.io_pump_exit_signal_sender.lock()
    }

    pub fn into_io_pump_handle(self) -> JoinHandle<Result<(), IOPumpError>> {
        self.io_pump_handle
    }
}

#[derive(Debug)]
pub enum IOPumpError {
    IOError(IOError),
    MpscChannelSendError(MpscSendError<RawPacket>),
    OneshotChannelRecvError(OneshotRecvError),
}

impl From<IOError> for IOPumpError {
    fn from(err: IOError) -> Self {
        Self::IOError(err)
    }
}

impl From<MpscSendError<RawPacket>> for IOPumpError {
    fn from(err: MpscSendError<RawPacket>) -> Self {
        Self::MpscChannelSendError(err)
    }
}

impl From<OneshotRecvError> for IOPumpError {
    fn from(err: OneshotRecvError) -> Self {
        Self::OneshotChannelRecvError(err)
    }
}

async fn pump_stream(
    id: usize,
    stream: TcpStream,
    mut tx_receiver: MpscReceiver<Bytes>,
    rx_sender: MpscSender<RawPacket>,
    mut exit_signal_receiver: OneshotReceiver<()>,
) -> Result<(), IOPumpError> {
    enum TxState {
        NoPacket,
        Sending(Bytes, usize),
        Dropped,
    }
    enum RxState {
        ReceivingHeader(usize),
        ReceivingPayload(u16, BytesMut),
    }

    let mut tx_state = TxState::NoPacket;
    let mut rx_state = RxState::ReceivingHeader(0);
    let mut rx_header = [0u8; size_of::<u16>() * 2];
    let (mut reader, mut writer) = stream.into_split();

    loop {
        match (tx_state, rx_state) {
            (TxState::NoPacket, RxState::ReceivingHeader(mut rx_index)) => {
                tokio::select! {
                    _ = &mut exit_signal_receiver => return Ok(()),
                    packet = tx_receiver.recv() => {
                        match packet {
                            Some(packet) => {
                                tx_state = TxState::Sending(packet, 0);
                            }
                            None => {
                                tx_state = TxState::Dropped;
                            }
                        }

                        rx_state = RxState::ReceivingHeader(rx_index);
                    }
                    result = reader.read(&mut rx_header[rx_index..]) => {
                        let length = result?;

                        if length == 0 {
                            return Ok(());
                        }

                        rx_index += length;

                        if rx_index == rx_header.len() {
                            rx_state = RxState::ReceivingPayload(
                                u16::from_be_bytes((&rx_header[0..2]).try_into().unwrap()),
                                BytesMut::with_capacity(u16::from_be_bytes(
                                    (&rx_header[0..2]).try_into().unwrap(),
                                ) as _),
                            );
                        } else {
                            rx_state = RxState::ReceivingHeader(rx_index);
                        }

                        tx_state = TxState::NoPacket;
                    }
                }
            }
            (TxState::NoPacket, RxState::ReceivingPayload(rx_code, mut rx_payload)) => {
                tokio::select! {
                    _ = &mut exit_signal_receiver => return Ok(()),
                    packet = tx_receiver.recv() => {
                        match packet {
                            Some(packet) => {
                                tx_state = TxState::Sending(packet, 0);
                            }
                            None => {
                                tx_state = TxState::Dropped;
                            }
                        }

                        rx_state = RxState::ReceivingPayload(rx_code, rx_payload);
                    }
                    result = reader.read(rx_payload.as_mut()) => {
                        let length = result?;

                        if length == 0 {
                            return Ok(());
                        }

                        unsafe {
                            rx_payload.set_len(rx_payload.len() + length);
                        }

                        if rx_payload.len() == rx_payload.capacity() {
                            rx_sender.send(RawPacket::new(rx_code, rx_payload))?;
                            rx_state = RxState::ReceivingHeader(0);
                        } else {
                            rx_state = RxState::ReceivingPayload(rx_code, rx_payload);
                        }

                        tx_state = TxState::NoPacket;
                    }
                }
            }
            (TxState::Sending(tx_packet, mut tx_index), RxState::ReceivingHeader(mut rx_index)) => {
                tokio::select! {
                    _ = &mut exit_signal_receiver => return Ok(()),
                    result = writer.write(tx_packet.as_ref()) => {
                        let length = result?;

                        if length == 0 {
                            return Ok(());
                        }

                        tx_index += length;

                        if tx_index == tx_packet.len() {
                            tx_state = TxState::NoPacket;
                        } else {
                            tx_state = TxState::Sending(tx_packet, tx_index);
                        }

                        rx_state = RxState::ReceivingHeader(rx_index);
                    }
                    result = reader.read(&mut rx_header[rx_index..]) => {
                        let length = result?;

                        if length == 0 {
                            return Ok(());
                        }

                        rx_index += length;

                        if rx_index == rx_header.len() {
                            rx_state = RxState::ReceivingPayload(
                                u16::from_be_bytes((&rx_header[0..2]).try_into().unwrap()),
                                BytesMut::with_capacity(u16::from_be_bytes(
                                    (&rx_header[0..2]).try_into().unwrap(),
                                ) as _),
                            );
                        } else {
                            rx_state = RxState::ReceivingHeader(rx_index);
                        }

                        tx_state = TxState::Sending(tx_packet, tx_index);
                    }
                }
            }
            (
                TxState::Sending(tx_packet, mut tx_index),
                RxState::ReceivingPayload(rx_code, mut rx_payload),
            ) => {
                tokio::select! {
                    _ = &mut exit_signal_receiver => return Ok(()),
                    result = writer.write(tx_packet.as_ref()) => {
                        let length = result?;

                        if length == 0 {
                            return Ok(());
                        }

                        tx_index += length;

                        if tx_index == tx_packet.len() {
                            tx_state = TxState::NoPacket;
                        } else {
                            tx_state = TxState::Sending(tx_packet, tx_index);
                        }

                        rx_state = RxState::ReceivingPayload(rx_code, rx_payload);
                    }
                    result = reader.read(rx_payload.as_mut()) => {
                        let length = result?;

                        if length == 0 {
                            return Ok(());
                        }

                        unsafe {
                            rx_payload.set_len(rx_payload.len() + length);
                        }

                        if rx_payload.len() == rx_payload.capacity() {
                            rx_sender.send(RawPacket::new(rx_code, rx_payload))?;
                            rx_state = RxState::ReceivingHeader(0);
                        } else {
                            rx_state = RxState::ReceivingPayload(rx_code, rx_payload);
                        }

                        tx_state = TxState::NoPacket;
                    }
                }
            }
            (TxState::Dropped, RxState::ReceivingHeader(mut rx_index)) => {
                tokio::select! {
                    _ = &mut exit_signal_receiver => return Ok(()),
                    result = reader.read(&mut rx_header[rx_index..]) => {
                        let length = result?;

                        if length == 0 {
                            return Ok(());
                        }

                        rx_index += length;

                        if rx_index == rx_header.len() {
                            rx_state = RxState::ReceivingPayload(
                                u16::from_be_bytes((&rx_header[0..2]).try_into().unwrap()),
                                BytesMut::with_capacity(u16::from_be_bytes(
                                    (&rx_header[0..2]).try_into().unwrap(),
                                ) as _),
                            );
                        } else {
                            rx_state = RxState::ReceivingHeader(rx_index);
                        }

                        tx_state = TxState::Dropped;
                    }
                }
            }
            (TxState::Dropped, RxState::ReceivingPayload(rx_code, mut rx_payload)) => {
                tokio::select! {
                    _ = &mut exit_signal_receiver => return Ok(()),
                    result = reader.read(rx_payload.as_mut()) => {
                        let length = result?;

                        if length == 0 {
                            return Ok(());
                        }

                        unsafe {
                            rx_payload.set_len(rx_payload.len() + length);
                        }

                        if rx_payload.len() == rx_payload.capacity() {
                            rx_sender.send(RawPacket::new(rx_code, rx_payload))?;
                            rx_state = RxState::ReceivingHeader(0);
                        } else {
                            rx_state = RxState::ReceivingPayload(rx_code, rx_payload);
                        }

                        tx_state = TxState::Dropped;
                    }
                }
            }
        }
    }
}
