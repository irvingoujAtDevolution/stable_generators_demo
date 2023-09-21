use std::f32::consts::E;

use crate::{traits_mock::{MockSSPi, Error, GeneratorLocal}, kerberos_mock::MockKerberos, ntlm_mock::MockNtlm, generator::{Generator, self}, something::{Event, UserResponse}};



enum NegotiateState {
    TryKerbros(MockKerberos),
    TryNtlm(MockNtlm)
}

struct Negotiate{
    state:NegotiateState,
}

impl<'a> MockSSPi<'a> for Negotiate {
    fn operations_require_async_io(&'a mut self) ->  Generator<'a, Event,UserResponse,Result<u32,Error>> {
        let some_code_needs_to_be_executed = some_function(); // we don't want this to be execut twice

        match &mut self.state {
            NegotiateState::TryKerbros(kerb) => {
                // this might or migh not sucessful, if sucessful, there are operation needs to be done, but we returned the generator already,
                return kerb.operations_require_async_io();
            },
            NegotiateState::TryNtlm(ntlm) => {
                return ntlm.operations_require_async_io();
            },
        }
        let some_other_code_needs_executed = some_function(); // unreachable
    }
}

fn some_function() -> i32{
    0
}

fn possible_way_to_call_negotiate(mut negotiate:Negotiate){
    // let mut generator = &negotiate.operations_require_async_io();
    // let state = generator.start();
    // let result = loop{
    //     match state {
    //         crate::generator::GeneratorState::Suspended(request) => {
    //             generator.resume(UserResponse::SomeValue(32));
    //         },
    //         crate::generator::GeneratorState::Completed(value) => break value,
    //     }
    // };
    
    // if let Ok(value) = result {
    //     let mut generator = negotiate.operations_require_async_io(); // sspi init_context needs to be called multiple time
    //     let result = loop{
    //         match state {
    //             crate::generator::GeneratorState::Suspended(request) => {
    //                 generator.resume(UserResponse::SomeValue(32));
    //             },
    //             crate::generator::GeneratorState::Completed(value) => break value,
    //         }
    //     };
    // }else{
    // // if fails, we s

    // }
    
    todo!()
}

pub struct AnotherPossibleNegotiate{
    pub url:String
}
impl<'a> MockSSPi<'a> for AnotherPossibleNegotiate {
    fn operations_require_async_io(&'a mut self) ->  Generator<'a, Event,UserResponse,Result<u32,Error>> {
        GeneratorLocal::new(move |yield_point| async move {
            let some_value = some_function();
            let mut kerb = MockKerberos{
                yield_point,
                url:self.url.clone()
            };
            let mut ntlm = MockNtlm{};

            let result = match kerb.operations_require_async_io().await{
                Ok(res) => {
                    return Ok(res);
                },
                Err(err) => {
                    return ntlm.operations_require_async_io().await;
                },
            };

        })
    }
}
