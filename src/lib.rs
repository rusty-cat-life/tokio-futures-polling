use std::env;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

use futures::Future;
use reqwest::r#async::Client;

use chrono::Local;

use serde::{Deserialize, Serialize};

use tokio::prelude::*;
use tokio::timer::Interval;

use std::time::{Duration, Instant};

const API_KEY: &str = "YOUR_API_KEY";
const DEFAULT_INTERVAL_DURATION: u64 = 10;

#[derive(Serialize, Deserialize, Debug)]
struct Clouds {
    all: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Coord {
    lon: f64,
    lat: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Main {
    temp: f64,
    pressure: i64,
    humidity: i64,
    temp_min: f64,
    temp_max: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct WeatherResult {
    coord: Coord,
    weather: Vec<Weather>,
    base: String,
    main: Main,
    visibility: i64,
    wind: Wind,
    clouds: Clouds,
    dt: i64,
    sys: Sys,
    id: i64,
    name: String,
    cod: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sys {
    #[serde(rename = "type")]
    _type: i64,
    id: i64,
    message: f64,
    country: String,
    sunrise: i64,
    sunset: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    id: i64,
    main: String,
    description: String,
    icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Wind {
    speed: f64,
}

pub struct Config {
    pub duration: u64,
}

impl Config {
    pub fn new(mut args: env::Args) -> Config {
        args.next();

        let duration = match args.next() {
            Some(s) => {
                s.parse::<u64>()
                    .map(|v| {
                        println!("Interval Duration {} secs was set.", v);
                        v
                    })
                    .unwrap_or_else(|_| {
                        println!(
                            "Failed to Parse Interval Argument. Default Interval Duration ({} secs) was set.",
                            DEFAULT_INTERVAL_DURATION
                        );
                        DEFAULT_INTERVAL_DURATION
                    })
            },
            None => {
                println!(
                    "Default Interval Duration ({} secs) was set.",
                    DEFAULT_INTERVAL_DURATION
                );
                DEFAULT_INTERVAL_DURATION
            }
        };

        Config { duration }
    }
}

pub fn run(config: Config) {
    let task = Interval::new(Instant::now(), Duration::from_secs(config.duration))
        .for_each(|_| {
            tokio::spawn(poll());
            Ok(())
        })
        .map_err(|e| panic!("Interval errored; err={:?}", e));

    tokio::run(task);
}

fn poll() -> impl Future<Item = (), Error = ()> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?id=1850147&units=metric&lang=ja&appid={}",
        API_KEY
    );

    Client::new()
        .get(&url)
        .send()
        .and_then(|mut res| {
            println!("{}", res.status());

            res.json::<WeatherResult>()
        })
        .map_err(|err| println!("request error: {}", err))
        .map(|result| {
            let (location, weather, temp) =
                (result.name, &result.weather[0].main, result.main.temp);

            let timestamp = Local::now().to_rfc3339();
            let text = format!("{} {} {} {}\n", timestamp, location, weather, temp);

            if let Ok(file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("weather.log")
            {
                let mut writer = BufWriter::new(file);
                if let Err(e) = writer.write(text.as_bytes()) {
                    println!("Failed to write to Log: {}", e);
                }
            } else {
                println!("Failed to create OpenOptions.");
            }
        })
}