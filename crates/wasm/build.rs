use std::env;
use std::time::SystemTime;

fn main() {
    println!("cargo:rerun-if-env-changed=NTESIM_BUILD_STARTED_AT");
    println!("cargo:rerun-if-env-changed=NTESIM_ENGINE_SOURCE_SHA256");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    let build_started_at = match env::var("NTESIM_BUILD_STARTED_AT") {
        Ok(val) if !val.trim().is_empty() => val.trim().to_string(),
        _ => current_utc_iso8601(),
    };
    println!("cargo:rustc-env=NTESIM_WASM_BUILD_TIME={build_started_at}");
}

fn current_utc_iso8601() -> String {
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let seconds_of_day = (total_secs % 86400) as u32;
    let hours = seconds_of_day / 3600;
    let minutes = (seconds_of_day % 3600) / 60;
    let seconds = seconds_of_day % 60;

    let mut days = (total_secs / 86400) as i64;
    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1024 + doe / 1461 - doe / 142401) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}.{millis:03}Z")
}
