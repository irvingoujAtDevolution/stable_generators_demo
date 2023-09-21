use stable_generators_demo::{Event, UserResponse};
use stable_generators_demo::generator::GeneratorState;
use stable_generators_demo::negotiate_mock::AnotherPossibleNegotiate;
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
                        dbg!(url);
                        UserResponse::SomeValue(20)// use kerbros
                    }
                }
            }
            // The generator is in its final state, break out the execution loop
            GeneratorState::Completed(out) => break dbg!(out),
        };

        // Resume the generator
        state = generator.resume(dbg!(response));
    }.unwrap();

    assert_eq!(out,20);

    // user may call it multiple times, because init_security context needs to be called multiple times
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
                        dbg!(url);
                        UserResponse::SomeValue(101) // throw err if over 100, inside kerbros impl
                    }
                }
            }
            // The generator is in its final state, break out the execution loop
            GeneratorState::Completed(out) => break dbg!(out),
        };

        // Resume the generator
        state = generator.resume(dbg!(response));
    };

    assert!(out.is_ok());
    assert_eq!(out.unwrap(),10); // what ntlm returns
}
