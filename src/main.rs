// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

#![feature(plugin,try_from,proc_macro,custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate log;
extern crate slog_envlogger;
#[macro_use]
extern crate quick_error;
extern crate rustc_serialize;
extern crate clap;
#[macro_use]
extern crate rusted_cypher;
extern crate r2d2;
extern crate r2d2_cypher;
extern crate uuid;
extern crate emailaddress;
extern crate url;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod entities;
mod business;
mod adapters;
mod infrastructure;

fn main() {
    slog_envlogger::init().unwrap();
    infrastructure::cli::run();
}
