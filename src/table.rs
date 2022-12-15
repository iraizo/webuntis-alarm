use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Lesson {
    #[serde(deserialize_with = "from_date")]
    date: NaiveDate,
    #[serde(deserialize_with = "from_time", rename = "startTime")]
    start_time: NaiveTime,
    #[serde(deserialize_with = "from_time", rename = "endTime")]
    end_time: NaiveTime,
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
