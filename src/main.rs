use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let resp =
        reqwest::blocking::get("https://mese.webuntis.com/WebUntis/?school=Nixdorf_BK_Essen")?;

    if resp.status() == 200 {
        let session_id = resp.headers()["set-cookie"]
            .to_str()?
            .split(";")
            .collect::<Vec<&str>>()[0];

        let cookie = format!(
            "{}; {}",
            session_id, r#"schoolname="_bml4ZG9yZl9ia19lc3Nlbg==""#
        );
        println!("cookie: {}", cookie);
        let client = reqwest::blocking::Client::new();

        let mut login_params = HashMap::new();

        login_params.insert("school", "Nixdorf_BK_Essen");
        login_params.insert("j_username", "HI-22C");
        login_params.insert("j_password", "hnbk_KB_2022");
        login_params.insert("token", "");

        let security_check = client
            .post("https://mese.webuntis.com/WebUntis/j_spring_security_check")
            .header("Cookie", cookie)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Origin", "https://mese.webuntis.com")
            .header("X-CSRF-TOKEN", "2bead485-19c4-425b-b63b-eefa24253dd7")
            .header("Accept", "application/json")
            .form(&login_params)
            .send()?;

        let test: serde_json::Value = serde_json::from_str(&security_check.text().unwrap())?;
        println!("{:?}", test);
    }
    Ok(())
}
