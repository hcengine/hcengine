use async_trait::async_trait;
use std::{io::Error, net::SocketAddr};

use tokio::net::{TcpListener, TcpStream};
use webparse::Response;
use wenmeng::{Body, HttpTrait, ProtError, ProtResult, RecvRequest, RecvResponse, Server};

use crate::{core::worker, HcWorkerState};

use super::HttpReceiver;

struct Operate;

#[async_trait]
impl HttpTrait for Operate {
    async fn operate(&mut self, req: &mut RecvRequest) -> ProtResult<RecvResponse> {
        let mut builder = Response::builder().version(req.version().clone());
        builder = builder.header("content-type", "text/plain; charset=utf-8");
        builder
            .body(Body::new_text("Hello, World!".to_string()))
            .map_err(|e| ProtError::from(e))

        // let mut builder = Response::builder().version(req.version().clone());
        // let body = match &*req.url().path {
        //     "/plaintext" | "/" => {
        //         builder = builder.header("content-type", "text/plain; charset=utf-8");
        //         Body::new_text("Hello, World!".to_string())
        //     }
        //     "/post" => {
        //         let body = req.body_mut();
        //         // let mut buf = [0u8; 10];
        //         // if let Ok(len) = body.read(&mut buf).await {
        //         //     println!("skip = {:?}", &buf[..len]);
        //         // }
        //         let mut binary = BinaryMut::new();
        //         body.read_all(&mut binary).await.unwrap();
        //         println!("binary = {:?}", binary);

        //         builder = builder.header("content-type", "text/plain");
        //         Body::new_binary(binary)
        //         // format!("Hello, World! {:?}", TryInto::<String>::try_into(binary)).to_string()
        //     }
        //     "/json" => {
        //         builder = builder.header("content-type", "application/json");
        //         #[derive(Serialize)]
        //         struct Message {
        //             message: &'static str,
        //         }
        //         Body::new_text(
        //             serde_json::to_string(&Message {
        //                 message: "Hello, World!",
        //             })
        //             .unwrap(),
        //         )
        //     }
        //     "/file" => {
        //         builder = builder.header("content-type", "application/json");
        //         let file = File::open("README.md").await?;
        //         let length = file.metadata().await?.len();
        //         Body::new_file(file, length)
        //     }
        //     _ => {
        //         builder = builder.status(404);
        //         Body::empty()
        //     }
        // };

        // let response = builder
        //     // .header("Content-Length", length as usize)
        //     .header(HeaderName::TRANSFER_ENCODING, "chunked")
        //     .body(body)
        //     .map_err(|_err| io::Error::new(io::ErrorKind::Other, ""))?;
        // Ok(response)
    }
}

pub struct HttpServer {
    id: u64,
    service_id: u32,
    worker: HcWorkerState,
}

impl HttpServer {
    pub fn new(id: u64, service_id: u32, worker: HcWorkerState) -> Self {
        Self {
            id,
            service_id,
            worker,
        }
    }
    
    pub async fn build_server(&self, stream: TcpStream, addr: SocketAddr) -> Server<TcpStream> {
        let mut server = Server::new(stream, Some(addr));
        server.set_callback_http(Box::new(Operate));
        return server;
    }

    pub async fn run_http(self, server: TcpListener, receiver: HttpReceiver) -> Result<(), ProtError> {
        tokio::spawn(async move {
            while let Ok((stream, addr)) = server.accept().await {
                let mut server = self.build_server(stream, addr).await;
                tokio::spawn(async move {
                    let _ret = server.incoming().await;
                });
            }
        });
        Ok(())
    }
}

// async fn process(stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
//     let mut server = Server::new(stream, Some(addr));
//     server.set_callback_http(Box::new(Operate));
//     let _ret = server.incoming().await;
//     Ok(())
// }

// #[tokio::main]
// async fn main() -> ProtResult<()> {
//     env_logger::init();
//     let addr = env::args()
//         .nth(1)
//         .unwrap_or_else(|| "0.0.0.0:8080".to_string());
//     let server = TcpListener::bind(&addr).await?;
//     println!("Listening on: {}", addr);
//     loop {
//         let (stream, addr) = server.accept().await?;
//         tokio::spawn(async move {
//             if let Err(e) = process(stream, addr).await {
//                 println!("failed to process connection; error = {}", e);
//             }
//         });
//     }
// }
