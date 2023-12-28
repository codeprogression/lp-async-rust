use std::collections::VecDeque;
use xactor::{Actor, Context, Handler, Message};
use crate::actors::{Report, TailRequest};

pub struct BufferSink {
    buffer: VecDeque<Report>
}

impl BufferSink{
    pub fn new(buffer_size: usize) -> Self {
        Self{
            buffer: VecDeque::with_capacity(buffer_size),
        }
    }
}

#[async_trait::async_trait]
impl Actor for BufferSink {
    async fn started(&mut self, ctx: &mut Context<Self>) -> xactor::Result<()> {
        ctx.subscribe::<Report>().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<Report> for BufferSink {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Report) {
        if self.buffer.len() == self.buffer.capacity(){
            self.buffer.pop_front();
        }
        self.buffer.push_back(msg);
    }
}


#[async_trait::async_trait]
impl Handler<TailRequest> for BufferSink {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: TailRequest) -> <TailRequest as Message>::Result {
        self.buffer.make_contiguous().iter().cloned().take(msg.0).collect::<Vec<Report>>()
    }
}