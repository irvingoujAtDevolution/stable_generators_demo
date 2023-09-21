use stable_generators_demo::client::{DemoClient, DemoHandler};
use stable_generators_demo::generator::{GeneratorState, self};
use stable_generators_demo::negotiate_mock::AnotherPossibleNegotiate;
use stable_generators_demo::something::{DoSomethingStruct, Event, UserResponse};
use stable_generators_demo::traits_mock::MockSSPi;


fn main() {
    let mut another_mock_nego = AnotherPossibleNegotiate{
        url:"test".to_string()
    };     
    let mut generator = another_mock_nego.operations_require_async_io();
    let mut state = generator.start();
    let out = loop {
        let response = match state {
            // The generator is suspended, handle the yielded value
            GeneratorState::Suspended(event) => {
                // How the events are actually handled is up to the caller (could perform I/O with or without async)
                match event {
                    Event::HttpRequest { url } => {
                        dbg!("http request called");
                        UserResponse::Payload(vec![1, 2, 3])
                    }
                    Event::PayloadLen(len) => {
                        dbg!("payload request called");
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

}
// Example of usage of our generator-based library, no "async" nor "await" in view

// fn main() {
//     dbg!("Entering main");

//     let input_url = "https://devolutions.gateway.ninja:8888/KdcProxy";
//     let client = DemoClient { yield_point: None };
//     let mut some_struct = DoSomethingStruct::new(Box::new(client));
//     let mut generator = some_struct.do_something(input_url.to_owned());
//     let mut state = generator.start();
//     let out = loop {
//         let response = match state {
//             // The generator is suspended, handle the yielded value
//             GeneratorState::Suspended(event) => {
//                 // How the events are actually handled is up to the caller (could perform I/O with or without async)
//                 match event {
//                     Event::HttpRequest { url } => {
//                         // assert_eq!(url, input_url);
//                         UserResponse::Payload(vec![1, 2, 3])
//                     }
//                     Event::PayloadLen(len) => {
//                         // assert_eq!(len, 3);
//                         UserResponse::SomeValue(u32::try_from(len).unwrap())
//                     }
//                 }
//             }
//             // The generator is in its final state, break out the execution loop
//             GeneratorState::Completed(out) => break dbg!(out),
//         };

//         // Resume the generator
//         state = generator.resume(dbg!(response));
//     };

//     // assert_eq!(out, 3);
//     dbg!(out);
// }





#[cfg(test)]
mod tests {
    use stable_generators_demo::{client::DemoClient, something::{DoSomethingStruct, Event, UserResponse}, generator::GeneratorState,generator::{Generator, self}, traits_mock::YieldPointLocal, negotiate_mock::AnotherPossibleNegotiate};

    #[test]
    fn create_yield_point_out_side() {
        dbg!("Entering main");

        let input_url = "https://devolutions.gateway.ninja:8888/KdcProxy";
        let client = DemoClient { yield_point: None };
        let mut some_struct = DoSomethingStruct::new(Box::new(client));

        let yield_point = YieldPointLocal::new();
        let mut generator = some_struct.do_something_another_way(input_url.to_owned(), yield_point);
        let mut state = generator.start();
        let out = loop {
            let response = match state {
                // The generator is suspended, handle the yielded value
                GeneratorState::Suspended(event) => {
                    // How the events are actually handled is up to the caller (could perform I/O with or without async)
                    match event {
                        Event::HttpRequest { url } => {
                            UserResponse::Payload(vec![1, 2, 3])
                        }
                        Event::PayloadLen(len) => {
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
        dbg!(out);
    }

    #[test]
    fn create_yield_point_at_beginning() {
        dbg!("Entering main");

        let input_url = "https://devolutions.gateway.ninja:8888/KdcProxy";
        let yield_point = YieldPointLocal::new();
        let client = DemoClient { yield_point: Some(yield_point.clone()) };
        let mut some_struct = DoSomethingStruct::new(Box::new(client));
        let mut generator = some_struct.do_something_another_way(input_url.to_owned(), yield_point);
        let mut state = generator.start();
        let out = loop {
            let response = match state {
                // The generator is suspended, handle the yielded value
                GeneratorState::Suspended(event) => {
                    // How the events are actually handled is up to the caller (could perform I/O with or without async)
                    match event {
                        Event::HttpRequest { url } => {
                            dbg!(url);
                            UserResponse::Payload(vec![1, 2, 3])
                        }
                        Event::PayloadLen(len) => {
                            dbg!(len);
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
    
        dbg!(out);
    }

}