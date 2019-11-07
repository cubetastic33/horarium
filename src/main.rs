#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate prettytable;
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

use postgres::{Connection, TlsMode};
use rocket::{
    Config, State, response::NamedFile,
};
use rocket_contrib::{serve::StaticFiles, templates::Template};
use std::{env, sync::Mutex};

mod webscrape;
mod db_operations;

use db_operations::*;

#[derive(Serialize)]
struct ClassDetails {
    name: String,
    url: String,
}

#[derive(Serialize)]
struct Context {
    classes: Vec<Vec<ClassDetails>>,
}

#[derive(Serialize, Clone)]
pub struct Timetable {
    day: String,
    classes: Vec<(String, String)>,
}

#[derive(Serialize)]
pub struct TimetableList {
    class_name: String,
    timetables: Vec<Timetable>,
}

#[derive(Serialize)]
pub struct Logs {
    logs: Vec<(String, String, String)>,
}

impl Default for Logs {
    fn default() -> Self {
        Logs {
            logs: Vec::new(),
        }
    }
}

pub fn class_url_to_name(url: &str) -> String {
    String::from(match url {
        "xia1" => "XI A₁",
        "xia2" => "XI A₂",
        "xia3" => "XI A₃",
        "xia4" => "XI A₄",
        "xiz1" => "XI Z₁",
        "xiz2" => "XI Z₂",
        "xiesp" => "XI ESP",
        x => x,
    })
}

#[get("/")]
fn index_route() -> Template {
    Template::render("index", Context {
        classes: vec![
            vec![ClassDetails {
                name: String::from("XI A₁"),
                url: String::from("/xia1")
            },
            ClassDetails {
                name: String::from("XI A₂"),
                url: String::from("/xia2")
            }],
            vec![ClassDetails {
                name: String::from("XI A₃"),
                url: String::from("/xia3")
            },
            ClassDetails {
                name: String::from("XI A₄"),
                url: String::from("/xia4")
            }],
            vec![ClassDetails {
                name: String::from("XI Z₁"),
                url: String::from("/xiz1")
            },
            ClassDetails {
                name: String::from("XI Z₂"),
                url: String::from("/xiz2")
            }],
            vec![ClassDetails {
                name: String::from("XI ESP"),
                url: String::from("/xiesp")
            },
            ClassDetails {
                name: String::from("FAQ"),
                url: String::from("#faq"),
            }],
        ],
    })
}

#[get("/<class_url>")]
fn get_timetable_page(conn: State<Mutex<Connection>>, class_url: String) -> Result<Template, failure::Error> {
    if ["xia2", "xia3", "xia4", "xiz1", "xiz2"].contains(&class_url.as_ref()) {
        return Ok(Template::render("not_available", TimetableList {
            class_name: class_url_to_name(&class_url),
            timetables: Vec::new(),
        }));
    }
    Ok(Template::render("timetable", get_timetables(&conn.lock().unwrap(), class_url)?))
}

#[get("/install_pwa")]
fn get_install_pwa_page() -> Template {
    Template::render("install_pwa", Context { classes: Vec::new() })
}

#[get("/logs")]
fn get_logs_page(conn: State<Mutex<Connection>>) -> Result<Template, failure::Error> {
    Ok(Template::render("logs", get_logs(&conn.lock().unwrap())?))
}

#[get("/application_error")]
fn get_application_error_page() -> Template {
    Template::render("application_error", Context { classes: Vec::new() })
}

#[get("/offline")]
fn get_offline_page() -> Template {
    Template::render("offline", Context { classes: Vec::new() })
}

#[get("/manifest.json")]
fn get_manifest_json() -> NamedFile {
    NamedFile::open("static/manifest.json").unwrap()
}

#[get("/service-worker.js")]
fn get_service_worker_js() -> NamedFile {
    NamedFile::open("static/service-worker.js").unwrap()
}

// POST requests

#[post("/refetch/<class_url>")]
fn refetch_route(conn: State<Mutex<Connection>>, class_url: String) -> String {
    forcibly_fetch(&conn.lock().unwrap(), class_url)
}

fn configure() -> Config {
    // Configure Rocket to serve on the port requested by Heroku.
    let mut config = Config::active().expect("could not load configuration");
    config
        .set_secret_key("<secret key>")
        .unwrap();
    if let Ok(port_str) = env::var("PORT") {
        let port = port_str.parse().expect("could not parse PORT");
        config.set_port(port);
    }
    config
}

fn rocket() -> rocket::Rocket {
    rocket::custom(configure())
        .mount(
            "/",
            routes![
                index_route,
                get_timetable_page,
                get_install_pwa_page,
                get_logs_page,
                get_application_error_page,
                get_offline_page,
                get_manifest_json,
                get_service_worker_js,
                refetch_route,
            ],
        )
        .mount("/styles", StaticFiles::from("static/styles"))
        .mount("/scripts", StaticFiles::from("static/scripts"))
        .mount("/fonts", StaticFiles::from("static/fonts"))
        .mount("/images", StaticFiles::from("static/images"))
        .attach(Template::fairing())
}

fn main() -> Result<(), failure::Error> {
    let client = Connection::connect("<URL>", TlsMode::None).unwrap();
    rocket().manage(Mutex::new(client)).launch();

    Ok(())
}
