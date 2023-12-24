use std::time::Duration;

use xactor::{Actor, Broker, Context, Handler, message, Service};
use crate::actors::FetchAll;

pub struct Looper;


#[async_trait::async_trait]
impl Actor for Looper {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()>  {
        let _ = ctx.address().send(Next);
        let _ = ctx.send_interval(Next, Duration::from_secs(30));
        Ok(())
    }
}

#[message]
#[derive(Clone)]
pub struct Next;

#[async_trait::async_trait]
impl Handler<Next> for Looper {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: Next)  {
        let _ = Broker::from_registry().await.unwrap().publish(FetchAll);
    }
}

