mod actions;
mod attributes;
mod components;
mod decrypt;
mod ibs;
mod js_functions;
mod mime;
mod types;

use crate::components::index::Index;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    yew::start_app::<Index>();
}
