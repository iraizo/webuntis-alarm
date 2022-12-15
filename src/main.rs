#![feature(str_split_as_str)]
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{Local, NaiveDate, NaiveTime};
use clap::Parser;
use config::Configuration;
use dotenvy::dotenv;
use reqwest::cookie::Cookie;
use serde::{Deserialize, Deserializer, Serialize};
use service::UntisService;
use table::Lesson;

pub mod config;
pub mod service;
pub mod table;

async fn send_table(req: HttpRequest, data: web::Data<Arc<Mutex<Vec<Lesson>>>>) -> impl Responder {
    let u_data = data.lock().unwrap();
    return HttpResponse::Ok().body(serde_json::to_string(&*u_data).unwrap());
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    pretty_env_logger::init();

    let config = Configuration::parse();
    let lessons = Arc::new(Mutex::new(vec![]));
    let untis_service = UntisService::new(config, lessons.clone());

    tokio::spawn(async move {
        loop {
            untis_service.clone().retrieve().await.unwrap();
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .route("/table", web::get().to(send_table))
            .app_data(web::Data::new(lessons.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
