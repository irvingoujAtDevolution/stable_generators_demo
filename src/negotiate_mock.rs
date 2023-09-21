use crate::{traits_mock::{MockSSPi, Error, GeneratorLocal}, kerberos_mock::MockKerberos, ntlm_mock::MockNtlm, generator::{Generator, self}, Event, UserResponse};


pub struct AnotherPossibleNegotiate{
    pub url:String
}
impl<'a> MockSSPi<'a> for AnotherPossibleNegotiate {
    fn operations_require_async_io(&'a mut self) ->  Generator<'a, Event,UserResponse,Result<u32,Error>> {
        GeneratorLocal::new(move |yield_point| async move {
            let _some_value = some_random_function();
            let mut kerb = MockKerberos{
                yield_point,
                url:self.url.clone()
            };
            let mut ntlm = MockNtlm{};

             let final_res = match kerb.operations_require_async_io().await{
                Ok(res) => {
                    return Ok(res);
                },
                Err(_) => {
                    return ntlm.operations_require_async_io().await;
                },
            };

            println!("we need to do something here {}",some_random_function());
            return final_res;
        })
    }
}

fn some_random_function() -> u32{
    // just some random function that may exist in the code
    0
}
