use afire::Server;

use crate::app::App;

mod index;

pub fn attach(server: &mut Server<App>) {
    index::attach(server);
}
