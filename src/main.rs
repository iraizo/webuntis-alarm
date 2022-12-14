#![feature(str_split_as_str)]
use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Local, TimeZone, Utc};
use clap::Parser;
use config::Configuration;
use dotenvy::dotenv;
use reqwest::{
    cookie::Cookie,
    header::{self, HeaderValue},
};
use serde::{Deserialize, Deserializer, Serialize};

pub mod config;

#[derive(Serialize, Deserialize, Debug)]
struct Lesson {
    date: i64,
    #[serde(deserialize_with = "from_time", rename = "startTime")]
    start_time: DateTime<Utc>,
    #[serde(deserialize_with = "from_time", rename = "endTime")]
    end_time: DateTime<Utc>,
}

fn from_time<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = u32::deserialize(deserializer)?.to_string();

    if s.len() > 4 {
        let hh = &s[0..2];
        let mm = &s[2..4];
    }

    Ok(date)
}

fn main() -> anyhow::Result<()> {
    dotenv()?;
    let config = Configuration::parse();

    let resp = reqwest::blocking::get(config.url)?;

    if resp.status() == 200 {
        let cookies: Vec<Cookie> = resp.cookies().collect();

        let session_id = format!("{}={}", &cookies[0].name(), &cookies[0].value());
        let school_name = format!("{}={}", &cookies[1].name(), &cookies[1].value());

        let cookie = format!("{}; {}", session_id, school_name);
        let client = reqwest::blocking::Client::new();
        let mut login_params = HashMap::new();

        login_params.insert("school", "Nixdorf_BK_Essen");
        login_params.insert("j_username", "HI-22C");
        login_params.insert("j_password", "hnbk_KB_2022");
        login_params.insert("token", "");

        let security_check = client
            .post("https://mese.webuntis.com/WebUntis/j_spring_security_check")
            .header("Cookie", &cookie)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&login_params)
            .send()?;

        let resp: serde_json::Value = serde_json::from_str(&security_check.text().unwrap())?;

        if resp["state"] == "SUCCESS" {
            let api_token = client
                .get("https://mese.webuntis.com/WebUntis/api/token/new")
                .header("Cookie", &cookie)
                .header("Accept", "application/json, text/plain, */*")
                .send()?;

            let date = Local::now().format("%Y-%m-%d").to_string();

            let time_table = client
                .get(format!("https://mese.webuntis.com/WebUntis/api/public/timetable/weekly/data?elementType=1&elementId=2902&date={}&formatId=2", date))
                .header("Cookie", &cookie)
                .header("Accept", "application/json")
                .send()?;

            if time_table.status() == 200 {
                let json: serde_json::Value = serde_json::from_str(&time_table.text().unwrap())?;

                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &json["data"]["result"]["data"]["elementPeriods"]["2902"]
                    )
                    .unwrap()
                );

                let lessons: Vec<Lesson> = json["data"]["result"]["data"]["elementPeriods"]["2902"]
                    .as_array()
                    .unwrap()
                    .into_iter()
                    .map(|v| serde_json::from_value(v.clone()).unwrap())
                    .collect();

                println!("{:?}", lessons);
            }
        }
    }
    Ok(())
}
