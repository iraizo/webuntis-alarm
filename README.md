# Webuntis alarm
### This is a small little "middleware" / web server intended to connect to a webuntis timetable used in german schools which i wrote when i was bored in class.
### Its purpose is to retrieve the next day's first lesson to automatically set my alarm accordingly.


## Requirements
- [Rust ](https://rustup.rs/)(nightly)
- Preferably an open port for remote use
- Common sense

## Usage
### Getting the server  to run
1. Clone or download this project using `git clone github.com/iraizo/webuntis-alarm`
2. run `cargo build` inside the directory 
3. create an `.env` file in the root of the project like below
    ```env
    USER="USERNAME_TO_LOGIN"
    PASSWORD="PASSWORD_TO_LOGIN"
    URL="https://mese.webuntis.com/WebUntis/?school=WHATEVER"
    HOST="127.0.0.1:8080"
    ```
    To retrieve the url go onto [untis](https://mese.webuntis.com) and find your school, the url should match whats above other than `school?=*` being different.  
    Username and password should be the one you use with untis  
    HOST is in the `IP:PORT` format and is where the middleware is hosted (127.0.0.1:8080) would be running local only.  
4. run `cargo run --release`
### Accessing the data
You can either manually get the data by accessing the routes listed below
```
GET /tomorrow => returns the first class for tomorrow (date, start, end)
GET /week => returns an array of the current classes for the current week (date, start, end)
```
or use my already done [shortcut](https://www.icloud.com/shortcuts/ac52b16cc96645d18792fc4b738ad604) for iOS, you will be asked for the URL which would be `YOUR_HOST_HERE/tomorrow`

If you need any help setting it up or feature suggestions / bugs feel free to contact me / create an issue.
