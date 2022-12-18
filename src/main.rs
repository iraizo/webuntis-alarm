#![feature(str_split_as_str)]
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{Days, Local, Timelike};
use clap::Parser;
use config::Configuration;
use dotenvy::dotenv;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde_json::json;
use service::UntisService;
use std::cmp::Ordering;
use table::Lesson;

pub mod config;
pub mod service;
pub mod table;

async fn weekly(_req: HttpRequest, data: web::Data<Arc<Mutex<Vec<Lesson>>>>) -> impl Responder {
    let mut u_data = data.lock().unwrap();
    u_data.sort_by(|a, b| match a.start_time.cmp(&b.start_time) {
        Ordering::Less => Ordering::Less,
        Ordering::Equal => Ordering::Equal,
        Ordering::Greater => Ordering::Greater,
    });
    return HttpResponse::Ok().body(serde_json::to_string(&*u_data).unwrap());
}

async fn first_class(
    _req: HttpRequest,
    data: web::Data<Arc<Mutex<Vec<Lesson>>>>,
) -> impl Responder {
    let u_data = data.lock().unwrap();
    let lessons = &mut u_data.clone();
    let tomorrow = Local::now().checked_add_days(Days::new(1)).unwrap();

    let mut day_lessons = lessons
        .iter()
        .filter(|s| s.date == tomorrow.date_naive())
        .collect::<Vec<_>>();

    day_lessons.sort_by_key(|s| s.start_time.num_seconds_from_midnight());

    if day_lessons.len() == 0 {
        return HttpResponse::Ok().body(json!({"error": "No lessons for tomorrow"}).to_string());
    }

    return HttpResponse::Ok().body(serde_json::to_string(&day_lessons[0]).unwrap());
}

fn load_rustls_config(cfg: &Configuration) -> rustls::ServerConfig {
    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open(cfg.cert.clone()).unwrap());
    let key_file = &mut BufReader::new(File::open(cfg.key.clone()).unwrap());

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    pretty_env_logger::init();

    let config = Configuration::parse();
    let lessons = Arc::new(Mutex::new(vec![]));
    let untis_service = UntisService::new(config.clone(), lessons.clone());
    let ssl_config = load_rustls_config(&config);

    tokio::spawn(async move {
        loop {
            untis_service.clone().retrieve().await.unwrap();
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .route("/week", web::get().to(weekly))
            .route("/tomorrow", web::get().to(first_class))
            .app_data(web::Data::new(lessons.clone()))
    })
    .bind_rustls(config.host, ssl_config)?
    .run()
    .await?;

    Ok(())
}
