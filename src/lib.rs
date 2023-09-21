pub mod generator;
pub mod traits_mock;
pub mod kerberos_mock;
pub mod ntlm_mock;
pub mod negotiate_mock;
pub enum Event {
    HttpRequest { url: String },
}

// Data type our generator will accept back at interruption points

#[derive(Debug)]
pub enum UserResponse {
    SomeValue(u32),
}

