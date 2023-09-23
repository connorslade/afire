use afire::Server;

use crate::app::App;

mod get_paste;
mod post_paste;

pub fn attach(server: &mut Server<App>) {
    get_paste::attach(server);
    post_paste::attach(server);
}
