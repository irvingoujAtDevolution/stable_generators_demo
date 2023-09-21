use crate::{generator::{Generator, YieldPoint}, something::{Event, UserResponse}};


pub type GeneratorLocal<'a> = Generator<'a, Event,UserResponse,Result<u32,Error>>;
pub type YieldPointLocal = YieldPoint<Event,UserResponse>;

#[derive(Debug)]
pub struct Error;
pub trait MockSSPi<'a> {
    fn operations_require_async_io(&'a mut self) ->  GeneratorLocal<'a>;
}