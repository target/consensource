#![feature(plugin)]
#![feature(proc_macro_hygiene, decl_macro)]
// 'needless_pass_by_value' lint disabled due to an issue in Rocket
// https://github.com/SergioBenitez/Rocket/issues/294
#![allow(clippy::needless_pass_by_value)]

#[macro_use]
extern crate clap;
extern crate database as database_manager;
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate protobuf;
extern crate sawtooth_sdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate bcrypt;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
extern crate serde_json;
extern crate tokio_core;
extern crate uuid;
#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
extern crate lazy_static;
extern crate hyper_sse;

mod database;
mod errors;
mod fairings;
mod jwt;
mod paging;
mod route_handlers;

use database::init_pool;
use fairings::CORS;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use rocket::response::NamedFile;
use route_handlers::{
    agents, authorization, blockchain, blocks, certificates, cors, factories, health,
    organizations, requests, standards, standards_body,
};
use std::path::{Path, PathBuf};
use std::{env, io, process};

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("../client/public/index.html")
}

// The rank is set high, such that the api get precedence
#[get("/<file..>", rank = 10)]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/public/").join(file)).ok()
}

fn main() {
    let matches = clap_app!(cert_registry_rest_api =>
    (version: crate_version!())
    (about: "Cert Registry REST API")
    (@arg verbose: -v --verbose +multiple
     "increase output verbosity")
    (@arg connect: default_value("tcp://localhost:4004") -C --connect +takes_value
     "connection endpoint for validator")
    (@arg dbname: default_value("cert-registry") --dbname +takes_value
       "the name of the database")
    (@arg dbhost: default_value("localhost") --dbhost +takes_value
        "the host of the database")
    (@arg dbport: default_value("5432") --dbport +takes_value
        "the port of the database")
    (@arg dbuser: default_value("cert-registry") --dbuser +takes_value
        "the authorized user of the database")
    (@arg dbpass: default_value("cert-registry") --dbpass +takes_value
        "the authorized user's password for database access")
    )
    .get_matches();

    let validator_url = matches.value_of("connect").unwrap().to_string();

    let console_log_level;
    match matches.occurrences_of("verbose") {
        0 => console_log_level = LevelFilter::Warn,
        1 => console_log_level = LevelFilter::Info,
        2 => console_log_level = LevelFilter::Debug,
        3 | _ => console_log_level = LevelFilter::Trace,
    }

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({l:5.5})} | {({M}:{L}):20.20} | {m}{n}",
        )))
        .build();

    let config = match Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(console_log_level))
    {
        Ok(x) => x,
        Err(e) => {
            for err in e.errors().iter() {
                error!("Configuration error: {}", err.to_string());
            }
            process::exit(1);
        }
    };

    match log4rs::init_config(config) {
        Ok(_) => (),
        Err(e) => {
            error!("Configuration error: {}", e.to_string());
            process::exit(1);
        }
    }

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        matches.value_of("dbuser").unwrap(),
        matches.value_of("dbpass").unwrap(),
        matches.value_of("dbhost").unwrap(),
        matches.value_of("dbport").unwrap(),
        matches.value_of("dbname").unwrap()
    );

    let connection_pool = init_pool(database_url);

    let host = env::var("ROCKET_ADDRESS").unwrap_or_else(|_| "127.0.0.1".into());

    let port: u16 = match env::var("ROCKET_PORT")
        .ok()
        .unwrap_or_else(|| "8000".into())
        .parse()
    {
        Ok(port) => port,
        Err(_) => {
            error!("Bad port value {}", matches.value_of("port").unwrap());
            process::exit(1);
        }
    };

    let block_watcher = blocks::BlockWatcher::new(connection_pool.clone());
    let watcher_thread = blocks::WatcherThread::run(block_watcher, 250, &host, port + 1);

    let error = rocket::ignite()
        .register(catchers![
            errors::not_found,
            errors::service_unavailable,
            errors::internal_error
        ])
        .manage(connection_pool)
        .manage(validator_url)
        .mount(
            "/api",
            routes![
                cors::cors_users_route,
                cors::cors_users_auth_route,
                cors::cors_batches_route,
                agents::fetch_agent,
                agents::fetch_agent_with_head_param,
                agents::list_agents,
                agents::list_agents_with_params,
                authorization::create_user,
                authorization::update_user,
                authorization::authenticate,
                blockchain::submit_batches,
                blockchain::list_statuses,
                blocks::fetch_block,
                blocks::fetch_block_with_head_param,
                blocks::list_blocks,
                blocks::list_blocks_with_params,
                factories::fetch_factory,
                factories::fetch_factory_with_head_param,
                factories::list_factories,
                factories::list_factories_params,
                health::check,
                requests::fetch_request,
                requests::fetch_request_with_head_param,
                requests::list_requests,
                requests::list_request_with_params,
                organizations::fetch_organization,
                organizations::fetch_organization_with_params,
                organizations::list_organizations,
                organizations::list_organizations_with_params,
                certificates::fetch_certificate,
                certificates::fetch_certificate_with_head_param,
                certificates::list_certificates,
                certificates::list_certificates_with_params,
                standards::list_standards,
                standards::list_standards_with_params,
                standards_body::list_standards_belonging_to_org
            ],
        )
        .mount("/", routes![index, files])
        .attach(CORS())
        .launch();

    watcher_thread.join().unwrap();

    println!("Launch failed!: Error: {}", error);
    process::exit(1);
}
