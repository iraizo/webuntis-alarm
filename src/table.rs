use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Element {
    #[serde(rename = "type")]
    pub kind: u8,
    pub id: u32,
    #[serde(default)]
    state: String,
    #[serde(default, rename = "longName")]
    pub long_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lesson {
    #[serde(deserialize_with = "from_date")]
    pub date: NaiveDate,
    #[serde(deserialize_with = "from_time", rename = "startTime")]
    pub start_time: NaiveTime,
    #[serde(deserialize_with = "from_time", rename = "endTime")]
    pub end_time: NaiveTime,
    #[serde(skip_serializing)]
    pub elements: Vec<Element>,
    #[serde(rename = "studentGroup")]
    pub student_group: String,
    #[serde(skip_deserializing)]
    pub room: String,
    #[serde(rename = "cellState")]
    pub state: String,
}

fn from_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = u32::deserialize(deserializer)?;
    let s = s.to_string();
    let year = s[0..4].parse::<i32>().unwrap();
    let month = s[4..6].parse::<u32>().unwrap();
    let day = s[6..8].parse::<u32>().unwrap();
    Ok(NaiveDate::from_ymd_opt(year, month, day).unwrap())
}

fn from_time<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = u32::deserialize(deserializer)?;
    let s = s.to_string();

    if s.len() >= 4 {
        let hh = &s[0..2];
        let mm = &s[2..4];

        return Ok(NaiveTime::from_hms_opt(
            hh.parse::<u32>().unwrap(),
            mm.parse::<u32>().unwrap(),
            0,
        )
        .unwrap());
    } else {
        let hh = &s[0..1];
        let mm = &s[1..3];

        return Ok(NaiveTime::from_hms_opt(
            hh.parse::<u32>().unwrap(),
            mm.parse::<u32>().unwrap(),
            0,
        )
        .unwrap());
    }
}
