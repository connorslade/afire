use afire::{
    prelude::*,
    trace::{set_log_level, Level},
};

macro_rules! empty_middleware {
    ($($name:tt,)*) => {
        $(
            struct $name;
            impl Middleware for $name {}
        )*
    };
}

empty_middleware! {
    Middleware1,
    Middleware2,
    Middleware3,
    Middleware4,
    Middleware5,
}

fn main() {
    set_log_level(Level::Debug);
    let mut server = Server::<()>::new([127, 0, 0, 1], 8080);
    server.route(Method::ANY, "**", |_req| Response::new());

    Middleware1.attach(&mut server);
    Middleware2.attach(&mut server);
    Middleware3.attach(&mut server);
    Middleware4.attach(&mut server);
    Middleware5.attach(&mut server);

    server.start().unwrap();
}
