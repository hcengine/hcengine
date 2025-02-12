use bpaf::OptionParser;
use tokio::sync::mpsc::Sender;
use wmhttp::{Client, ClientOption, ProtError, ProtResult, RecvRequest, RecvResponse};

use crate::{HcMsg, HcWorkerState};

pub struct HttpClient {}

impl HttpClient {
    pub async fn do_request(
        sender: Sender<HcMsg>,
        service_id: u32,
        session: i64,
        req: RecvRequest,
        option: Option<ClientOption>,
    ) -> ProtResult<()> {
        match Self::inner_request(req, option).await {
            Ok(res) => {
                let _ = sender
                    .send(HcMsg::http_return(service_id, session, Some(res), None))
                    .await;
            }
            Err(e) => {
                let _ = sender
                    .send(HcMsg::http_return(
                        service_id,
                        session,
                        None,
                        Some(format!("{}", e)),
                    ))
                    .await;
            }
        }
        Ok(())
    }

    async fn inner_request(
        req: RecvRequest,
        option: Option<ClientOption>,
    ) -> ProtResult<RecvResponse> {
        println!("req === {}", req.url());
        let client = Client::builder()
            .option(option.unwrap_or(ClientOption::default()))
            .url(req.url().clone())?
            .connect()
            .await?;

        let (mut recv, _sender) = client.send2(req).await?;
        match recv.recv().await {
            Some(r) => {
                let mut res = r?;
                res.body_mut().wait_all().await;
                // let res = res.into_type::<String>();
                // println!("res = {}", res);
                // println!("res = {:?}", res.body());
                Ok(res)
            }
            None => Err(ProtError::read_timeout("not receiver response")),
        }
    }
}
