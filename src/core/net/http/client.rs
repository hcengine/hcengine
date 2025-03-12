use algorithm::buf::BinaryMut;
use tokio::sync::mpsc::{Sender, UnboundedSender};
use wmhttp::{Client, ClientOption, ProtError, ProtResult, RecvRequest, RecvResponse};

use crate::{wrapper::WrapperLuaMsg, Config, HcMsg, HcWorkerState, LuaMsg};

// TODO 缓存请求, 复用链接
pub struct HttpClient {}

impl HttpClient {
    pub async fn do_request(
        sender: UnboundedSender<HcMsg>,
        service_id: u32,
        session: i64,
        req: RecvRequest,
        option: Option<ClientOption>,
    ) -> ProtResult<()> {
        match Self::inner_request(req, option).await {
            Ok(res) => {
                let data = BinaryMut::new();
                let mut msg = LuaMsg {
                    ty: Config::TY_HTTP_RES,
                    sender: 0,
                    receiver: service_id,
                    sessionid: session,
                    err: None,
                    data,
                    ..Default::default()
                };
                msg.obj = Some(WrapperLuaMsg::response(res));
                let _ = sender.send(HcMsg::RespMsg(msg));
            }
            Err(e) => {
                let _ = sender.send(HcMsg::RespMsg(LuaMsg::new_error(
                    format!("{}", e),
                    service_id,
                    session,
                )));

                // let _ = sender
                //     .send(HcMsg::http_return(
                //         service_id,
                //         session,
                //         None,
                //         Some(format!("{}", e)),
                //     ))
                //     .await;
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
