#![feature(str_split_as_str)]
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{Datelike, Days, Local, NaiveDate, NaiveTime, Timelike};
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

async fn weekly(req: HttpRequest, data: web::Data<Arc<Mutex<Vec<Lesson>>>>) -> impl Responder {
    let u_data = data.lock().unwrap();
    return HttpResponse::Ok().body(serde_json::to_string(&*u_data).unwrap());
}

async fn first_class(req: HttpRequest, data: web::Data<Arc<Mutex<Vec<Lesson>>>>) -> impl Responder {
    let u_data = data.lock().unwrap();
    let lessons = &mut u_data.clone();
    let tomorrow = Local::now().checked_add_days(Days::new(1)).unwrap();

    let mut day_lessons = lessons
        .iter()
        .filter(|s| s.date == tomorrow.date_naive())
        .collect::<Vec<_>>();

    day_lessons.sort_by_key(|s| s.start_time.num_seconds_from_midnight());

    return HttpResponse::Ok().body(serde_json::to_string(&day_lessons[0]).unwrap());
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    pretty_env_logger::init();

    let config = Configuration::parse();
    let lessons = Arc::new(Mutex::new(vec![]));
    let untis_service = UntisService::new(config.clone(), lessons.clone());

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
    .bind(config.host)?
    .run()
    .await?;

    Ok(())
}
