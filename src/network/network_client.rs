use parking_lot::Mutex;
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::oneshot::Sender;
use tokio::task::JoinHandle;

pub struct NetworkClient {
	index: usize,
	termination_signal_sender: Sender<()>,
	write_task_join_handle: JoinHandle<()>,
	read_task_join_handle: JoinHandle<()>,
}

impl NetworkClient {
	pub fn new(
		index: usize,
		termination_signal_sender: Sender<()>,
		write_task_join_handle: JoinHandle<()>,
		read_task_join_handle: JoinHandle<()>,
	) -> NetworkClient {
		NetworkClient {
			index,
			termination_signal_sender,
			write_task_join_handle,
			read_task_join_handle,
		}
	}

	pub fn index(&self) -> usize {
		self.index
	}

	pub fn task_join_handle(self) -> (Sender<()>, JoinHandle<()>, JoinHandle<()>) {
		(
			self.termination_signal_sender,
			self.write_task_join_handle,
			self.read_task_join_handle,
		)
	}
}
