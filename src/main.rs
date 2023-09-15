use stable_generators_demo::generator::GeneratorState;
use stable_generators_demo::something::{do_something, Event, UserResponse};

// Example of usage of our generator-based library, no "async" nor "await" in view

fn main() {
    dbg!("Entering main");

    let input_url = "https://devolutions.gateway.ninja:8888/KdcProxy";
    let mut do_something_generator = do_something(input_url.to_owned());

    dbg!("Drive the generator");

    // Start the generator
    let mut do_something_state = do_something_generator.start();

    let out = loop {
        let response = match do_something_state {
            // The generator is suspended, handle the yielded value
            GeneratorState::Suspended(event) => {
                // How the events are actually handled is up to the caller (could perform I/O with or without async)
                match event {
                    Event::HttpRequest { url } => {
                        assert_eq!(url, input_url);
                        UserResponse::Payload(vec![1, 2, 3])
                    }
                    Event::PayloadLen(len) => {
                        assert_eq!(len, 3);
                        UserResponse::SomeValue(u32::try_from(len).unwrap())
                    }
                }
            }
            // The generator is in its final state, break out the execution loop
            GeneratorState::Completed(out) => break dbg!(out),
        };

        // Resume the generator
        do_something_state = do_something_generator.resume(dbg!(response));
    };

    assert_eq!(out, 3);
}
