use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use chrono::{Local, NaiveDate};
use reqwest::cookie::Cookie;

use crate::{
    config::Configuration,
    table::{Element, Lesson},
};

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
            let unobfuscated_school_name = school_split.as_str();
            let user = &self.config.user.as_str();
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
                "Security check response: {:#?}",
                serde_json::to_string_pretty(&resp)
            );

            if resp["state"] == "SUCCESS" {
                let mut date = Local::now().format("%Y-%m-%d").to_string();
                let mut s = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
                s = s.succ_opt().unwrap();
                date = s.format("%Y-%m-%d").to_string();

                let time_table = client
                .get(format!("https://mese.webuntis.com/WebUntis/api/public/timetable/weekly/data?elementType=1&elementId=2902&date={}&formatId=2", date))
                .header("Cookie", &cookie)
                .header("Accept", "application/json")
                .send().await.context("Failed to get time table for this week")?;

                if time_table.status() == 200 {
                    let json: serde_json::Value = serde_json::from_str(&time_table.text().await?)?;
                    let mut lessons: Vec<Lesson> = json["data"]["result"]["data"]["elementPeriods"]
                        [&json["data"]["result"]["data"]["elementIds"][0].to_string()]
                        .as_array()
                        .unwrap()
                        .into_iter()
                        .map(|v| serde_json::from_value(v.clone()).unwrap())
                        .collect();

                    let elements: Vec<Element> = json["data"]["result"]["data"]["elements"]
                        .as_array()
                        .unwrap()
                        .into_iter()
                        .map(|v| serde_json::from_value(v.clone()).unwrap())
                        .collect();

                    for lesson in &mut lessons {
                        for el in &lesson.elements {
                            if el.kind == 4 {
                                let room: Vec<&Element> =
                                    elements.iter().filter(|e| e.id == el.id).collect();
                                lesson.room = room[0].long_name.clone();
                            }
                        }
                    }

                    log::info!("{:#?}", lessons);

                    let mut table_mutex = self.lessons.lock().unwrap();
                    *table_mutex = lessons;

                    println!("set data");
                }
            }

            return Ok(());
        }
        Ok(())
    }
}
