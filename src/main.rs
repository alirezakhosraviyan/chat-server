use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::broadcast;


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8000").await.unwrap();
    let (tx, _) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let cloned_tx = tx.clone();
        let mut subscriber = tx.subscribe();

        tokio::spawn(async move {
            let (socket_reader_half, mut socket_writer_half) = socket.split();

            let mut reader = BufReader::new(socket_reader_half);
            let mut lines = String::new();

            loop {
                select! {
                    res = reader.read_line(&mut lines) => {
                        if res.unwrap() == 2 { break; }
                        cloned_tx.send((lines.clone(), addr)).unwrap();
                        lines.clear();
                    }
                    res = subscriber.recv() => {
                        let (msg, other_addr) = res.unwrap();

                        if addr != other_addr {
                            socket_writer_half.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }

                }
            }
        });
    }
}
