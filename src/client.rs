use crate::{
    generator::{Generator, GeneratorState, YieldPoint},
    something::{Event, UserResponse},
};
use async_trait::async_trait;

#[async_trait]
pub trait NetworkClient: Send + Sync{
    type YieldType: Unpin + Send;
    type ResumeType: Send;
    async fn send(&mut self, request: Self::YieldType) -> Self::ResumeType;
    fn box_clone(
        &mut self,
    ) -> Box<dyn NetworkClient<YieldType = Self::YieldType, ResumeType = Self::ResumeType>>;
    fn with_yield_point(
        &mut self,
        yield_point:YieldPoint<Self::YieldType, Self::ResumeType>,
    ) -> Box<dyn NetworkClient<YieldType = Self::YieldType, ResumeType = Self::ResumeType>>;
    fn add_yield_point(
        &mut self,
        yield_point:YieldPoint<Self::YieldType, Self::ResumeType>,
    );
}

pub struct DemoClient {
    pub yield_point: Option<YieldPoint<Event, UserResponse>>,
}



#[async_trait]
impl NetworkClient for DemoClient {
    type ResumeType = UserResponse;
    type YieldType = Event;
    async fn send(&mut self, event: Event) -> UserResponse {
        self.yield_point.as_mut().expect("
            not yield point found
        ").suspend(event).await
    }

    fn with_yield_point(
        &mut self,
        yield_point:YieldPoint<Self::YieldType, Self::ResumeType>,
    ) -> Box<dyn NetworkClient<YieldType = Self::YieldType, ResumeType = Self::ResumeType>>{
        self.yield_point = Some(yield_point);
        self.box_clone()
    }

    fn add_yield_point(
        &mut self,
        yield_point:YieldPoint<Self::YieldType, Self::ResumeType>,
    ){
        self.yield_point = Some(yield_point);
    }

    fn box_clone(
        &mut self,
    ) -> Box<dyn NetworkClient<YieldType = Self::YieldType, ResumeType = Self::ResumeType>> {
        if let Some(point) = &self.yield_point {
            let clone = point.clone();
            return Box::new(DemoClient {
                yield_point:Some(clone)
            });
        } else {
            return Box::new(DemoClient { yield_point: None });
        }
    }
}

pub trait GeneratorHandler<Y, R, O>: Send + Sync {
    fn handle(&mut self, generator: &mut Generator<Y, R, O>) -> O;
    fn box_clone(&self) -> Box<dyn GeneratorHandler<Y, R, O>>;
}

pub struct DemoHandler {}

impl GeneratorHandler<Event, UserResponse, u32> for DemoHandler {
    fn handle(&mut self, generator: &mut Generator<Event, UserResponse, u32>) -> u32 {
        let mut state = generator.start();
        let out = loop {
            let response = match state {
                // The generator is suspended, handle the yielded value
                GeneratorState::Suspended(event) => {
                    // How the events are actually handled is up to the caller (could perform I/O with or without async)
                    match event {
                        Event::HttpRequest { url } => {
                            // assert_eq!(url, input_url);
                            UserResponse::Payload(vec![1, 2, 3])
                        }
                        Event::PayloadLen(len) => {
                            // assert_eq!(len, 3);
                            UserResponse::SomeValue(u32::try_from(len).unwrap())
                        }
                    }
                }
                // The generator is in its final state, break out the execution loop
                GeneratorState::Completed(out) => break dbg!(out),
            };

            // Resume the generator
            state = generator.resume(dbg!(response));
        };

        // assert_eq!(out, 3);
        return out;
    }

    fn box_clone(&self) -> Box<dyn GeneratorHandler<Event, UserResponse, u32>> {
        let new = DemoHandler {};
        return Box::new(new);
    }
}
