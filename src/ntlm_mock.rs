use crate::{traits_mock::{MockSSPi, GeneratorLocal}, generator::{Generator, YieldPoint}};

pub struct MockNtlm;
impl<'a> MockSSPi<'a> for MockNtlm {
    fn operations_require_async_io(&mut self) ->  GeneratorLocal<'a> {
        Generator::new( move|yield_point| async move {
            Ok((10 as u32))
        })
    }
}