use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use chrono::Local;
use reqwest::cookie::Cookie;

use crate::{config::Configuration, table::Lesson};

#[derive(Clone)]
pub struct UntisService {
    config: Configuration,
    pub lessons: Arc<Mutex<Vec<Lesson>>>,
}

impl UntisService {
    pub fn new(config: Configuration, lessons: Arc<Mutex<Vec<Lesson>>>) -> Self {
        Self { config, lessons }
    }

    pub async fn retrieve(&self) -> anyhow::Result<()> {
        let resp = reqwest::get(&self.config.url)
            .await
            .context("Failed to retrieve school data")?;

        log::info!("Requested session: {:?}", resp.headers());

        if resp.status() == 200 {
            let cookies: Vec<Cookie> = resp.cookies().collect();

            let session_id = format!("{}={}", &cookies[0].name(), &cookies[0].value());
            let school_name = format!("{}={}", &cookies[1].name(), &cookies[1].value());

            let cookie = format!("{}; {}", session_id, school_name);
            let client = reqwest::Client::new();
            let mut login_params = HashMap::new();

            let school_split = &mut self.config.url.split('=');
            school_split.next();
            school_split.next();
            let unobfuscated_school_name = school_split.as_str();
            let user = &self.config.username.as_str();
            let pass = &self.config.password.as_str();

            login_params.insert("school", unobfuscated_school_name);
            login_params.insert("j_username", user);
            login_params.insert("j_password", pass);
            login_params.insert("token", &"");

            log::info!("Login parameters: {:?}", login_params);

            let security_check = client
                .post("https://mese.webuntis.com/WebUntis/j_spring_security_check")
                .header("Cookie", &cookie)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("Accept", "application/json")
                .form(&login_params)
                .send()
                .await
                .context("Failed to send security check")?;

            let resp: serde_json::Value = serde_json::from_str(&security_check.text().await?)?;

            log::info!(
                "Security check response: {:?}",
                serde_json::to_string_pretty(&resp)
            );

            if resp["state"] == "SUCCESS" {
                let date = Local::now().format("%Y-%m-%d").to_string();

                let time_table = client
                .get(format!("https://mese.webuntis.com/WebUntis/api/public/timetable/weekly/data?elementType=1&elementId=2902&date={}&formatId=2", date))
                .header("Cookie", &cookie)
                .header("Accept", "application/json")
                .send().await.context("Failed to get time table for this week")?;

                if time_table.status() == 200 {
                    let json: serde_json::Value = serde_json::from_str(&time_table.text().await?)?;

                    let mut table_mutex = self.lessons.lock().unwrap();

                    let tables: Vec<Lesson> = json["data"]["result"]["data"]["elementPeriods"]
                        ["2902"]
                        .as_array()
                        .unwrap()
                        .into_iter()
                        .map(|v| serde_json::from_value(v.clone()).unwrap())
                        .collect();

                    *table_mutex = tables;
                    println!("set data");
                }
            }

            return Ok(());
        }
        Ok(())
    }
}
