use crate::{traits_mock::{MockSSPi, YieldPointLocal, GeneratorLocal, Error}, generator::Generator, something::{Event, UserResponse}};


pub struct MockKerberos {
    pub yield_point:YieldPointLocal,
    pub url:String
}
unsafe impl Sync for MockKerberos {}
unsafe impl Send for MockKerberos {}

impl<'a> MockSSPi<'a> for MockKerberos {
    fn operations_require_async_io(&'a mut self) ->  Generator<'a, Event,UserResponse,Result<u32,Error>> {
        let mut generator = GeneratorLocal::new_empty();
        let point_clone = self.yield_point.clone();
        generator.accept_yield_point(&point_clone);

        generator.accept_task(async move {
            self.do_the_thing().await
        });

        generator
    }
}

impl MockKerberos {
    async fn do_the_thing(&self)->Result<u32,Error>{
        let res = self.yield_point.suspend(Event::HttpRequest { url: self.url.clone() }).await;
        match res {
            UserResponse::Payload(payload) => {
                Err(Error)
            },
            UserResponse::SomeValue(v) => {
                Ok(v)
            },
        }
    }

    pub fn new(url:String)->Self{
        let yield_point = YieldPointLocal::new();
        Self { yield_point,url }
    }
}
