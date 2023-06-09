#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{json::Json, Serialize};
use rocket::tokio::time::{interval, Duration};
use rocket::State;

use rocket_slogger::Slogger;

mod hardware;
use hardware::{read_sensor, setup_sensor, LockableSensor};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct SensorResponse {
    id: String,
    state: String,
    value: f32,
}

fn lux_to_response(lux: f32) -> SensorResponse {
    SensorResponse {
        id: "sensor-ambient_light".to_string(),
        state: format!("{lux} lx").to_string(),
        value: lux,
    }
}

#[get("/sensor/ambient_light")]
fn ambient_light(
    managed_sensor: &State<LockableSensor>,
) -> Result<Json<SensorResponse>, Custom<String>> {
    match read_sensor(&managed_sensor) {
        Ok(lux) => Ok(Json(lux_to_response(lux))),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[get("/events")]
async fn events(managed_sensor: &State<LockableSensor>) -> EventStream![Event + '_] {
    EventStream! {
        let mut interval = interval(Duration::from_secs(2));
        loop {
            match read_sensor(&managed_sensor) {
                Ok(lux) => yield Event::json(&lux_to_response(lux)).event("state"),
                Err(e) => yield Event::data(format!("Error: {}", e))
            }
            interval.tick().await;
        }
    }
}

#[get("/logs")]
fn logs() -> Result<String, Custom<String>> {
    match std::fs::read_to_string("/var/log/lunarsensor/lunarsensor") {
        Ok(v) => Ok(v),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

#[launch]
fn rocket() -> _ {
    let log_fairing = Slogger::new_terminal_logger();

    let lockable_sensor = match setup_sensor() {
        Ok(v) => v,
        Err(e) => {
            println!("Sensor not initialized: {}", e);
            panic!();
        }
    };
    rocket::build()
        .attach(log_fairing)
        .manage(lockable_sensor)
        .mount("/", routes![ambient_light])
        .mount("/", routes![events])
        .mount("/", routes![logs])
}
