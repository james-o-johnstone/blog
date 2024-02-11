use regex::Regex;
use wasm_bindgen::prelude::*;

const PACE_CONVERSION_FACTOR: f64 = 1.60934;

#[wasm_bindgen]
pub fn convert_pace(pace: &str, unit: &str) -> Result<JsValue, JsError> {
    let pace_mins = parse_pace(pace).map_err(|_| JsError::new("Invalid format, expect mm:ss"))?;

    let result: f64;
    let output_unit: &str;
    if unit == "kms" {
        result = pace_mins * PACE_CONVERSION_FACTOR;
        output_unit = "per mile";
    } else {
        result = pace_mins / PACE_CONVERSION_FACTOR;
        output_unit = "per km"
    }

    let converted_mins = result.trunc();
    let converted_secs = (result.fract() * 60.0).trunc();

    return Result::Ok(JsValue::from_str(&format!("{converted_mins}:{converted_secs:02} {output_unit}")));
}

#[wasm_bindgen]
pub fn calculate_distance(time: &str, pace: &str, unit: &str) -> Result<JsValue, JsError> {
    let pace_mins = parse_pace(pace).map_err(|_| JsError::new("Invalid format, expect mm:ss"))?;
    let time_mins = parse_time(time).map_err(|_| JsError::new("Invalid format, expect hh:mm:ss"))?;
    let distance = time_mins / pace_mins;
    return Result::Ok(JsValue::from_str(&format!("{distance:.2} {unit}")));
}

struct ParseError;
fn parse_pace(pace: &str) -> Result<f64, ParseError> {
    let valid_pace = Regex::new(r"^[0-5]?\d$|^[0-5]?\d:[0-5]\d$").unwrap();
    if !valid_pace.is_match(pace) {
        return Result::Err(ParseError);
    }

    let total_mins: f64;
    if !pace.contains(":") {
        total_mins = pace.parse().unwrap();
    } else {
        let parts = pace.split(":").collect::<Vec<&str>>();
        let minutes: f64 = parts[0].parse().unwrap();
        let seconds: f64 = parts[1].parse().unwrap();
        total_mins = minutes + seconds/60.0;
    }
    return Result::Ok(total_mins);
}

fn parse_time(time: &str) -> Result<f64, ParseError> {
    let valid_time = Regex::new(r"^\d{2}:[0-5]\d:[0-5]\d$").unwrap();
    if !valid_time.is_match(time) {
        return Result::Err(ParseError);
    }

    let parts = time.split(":").collect::<Vec<&str>>();
    let hours: f64 = parts[0].parse().unwrap();
    let minutes: f64 = parts[1].parse().unwrap();
    let seconds: f64 = parts[2].parse().unwrap();
    let total_mins = hours*60.0 + minutes + seconds/60.0;
    return Result::Ok(total_mins);
}
