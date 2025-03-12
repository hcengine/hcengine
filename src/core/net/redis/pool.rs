use std::{
    collections::{HashMap, LinkedList},
    io,
    ops::Not,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc, Mutex,
    },
};

use algorithm::buf::BinaryMut;
use redis::{
    aio::{Connection, ConnectionLike, MultiplexedConnection},
    cluster::ClusterClient,
    cluster_async::ClusterConnection,
    Client, ErrorKind, Msg, PushKind, RedisError, RedisResult, Value,
};
use tokio::sync::{
    mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender},
    Notify,
};

use super::RedisMsg;
use crate::{
    core::worker, wrapper::WrapperLuaMsg, Config, HcMsg, HcNodeState, HcWorkerState, LuaMsg,
};
use futures_util::{FutureExt, Stream, StreamExt};

struct Inner {
    pub client_caches: Mutex<LinkedList<MultiplexedConnection>>,
}

#[derive(Clone)]
pub struct RedisPool {
    pub client: Client,
    inner: Arc<Inner>,
}

pub struct RedisGetConn {
    pub pool: Arc<RedisPool>,
}

unsafe impl Send for RedisGetConn {}
unsafe impl Sync for RedisGetConn {}

pub struct PoolClient {
    pub pool: Arc<RedisPool>,
    pub client: Option<MultiplexedConnection>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            client_caches: Mutex::new(LinkedList::new()),
        }
    }
}

impl RedisPool {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            inner: Arc::new(Inner::new()),
        }
    }

    pub fn get_conn(&self) -> RedisGetConn {
        RedisGetConn {
            pool: Arc::new(self.clone()),
        }
    }

    pub async fn get_client(&self) -> RedisResult<PoolClient> {
        {
            let mut l = self.inner.client_caches.lock().unwrap();
            if !l.is_empty() {
                let client = l.pop_front().unwrap();
                return Ok(PoolClient {
                    client: Some(client),
                    pool: Arc::new(self.clone()),
                })
            }
        }
        let client = self.client.get_multiplexed_async_connection().await?;
        Ok(PoolClient {
            pool: Arc::new(self.clone()),
            client: Some(client),
        })
    }

    pub fn recycle(&self, client: MultiplexedConnection) {
        let mut l = self.inner.client_caches.lock().unwrap();
        l.push_back(client);
        println!("recycle redis client now len = {:?}", l.len());
    }
}

impl RedisGetConn {
    pub async fn get(&self) -> RedisResult<PoolClient> {
        self.pool.get_client().await
    }
}

impl Drop for PoolClient {
    fn drop(&mut self) {
        if let Some(c) = self.client.take() {
            self.pool.recycle(c);
        }
    }
}

impl ConnectionLike for PoolClient {
    fn req_packed_command<'a>(&'a mut self, cmd: &'a redis::Cmd) -> redis::RedisFuture<'a, Value> {
        if let Some(c) = &mut self.client {
            c.req_packed_command(cmd)
        } else {
            async {
                Err(RedisError::from((
                    ErrorKind::ClientError,
                    "client now is not availd",
                    "".to_string(),
                )))
            }
            .boxed()
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<Value>> {
        if let Some(c) = &mut self.client {
            c.req_packed_commands(cmd, offset, count)
        } else {
            async {
                Err(RedisError::from((
                    ErrorKind::ClientError,
                    "client now is not availd",
                    "".to_string(),
                )))
            }
            .boxed()
        }
    }

    fn get_db(&self) -> i64 {
        self.client.as_ref().map(|v| v.get_db()).unwrap_or(0)
    }
}
