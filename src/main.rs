use std::{env, process::exit, thread, time::Duration};
use chrono::{DateTime, Utc};
use cron::Schedule;
use std::str::FromStr;

struct Job {
    method: String,
    url: String,
    schedule: Schedule,
    next_fire: DateTime<Utc>,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

fn env_or_exit(key: &str) -> String {
    match env::var(key) {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("Missing required env var: {}", key);
            exit(1);
        }
    }
}

fn parse_headers(s: &str) -> Vec<(String, String)> {
    if s.trim().is_empty() {
        return Vec::new();
    }
    s.split(',')
        .filter_map(|pair| {
            let (k, v) = pair.split_once(':')?;
            let k = k.trim();
            let v = v.trim();
            if k.is_empty() {
                return None;
            }
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}

fn split_jobs(spec: &str) -> impl Iterator<Item = &str> {
    spec.split(|c| c == ';' || c == '\n' || c == '\r')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
}

fn parse_jobs(spec: &str) -> Vec<Job> {
    let mut v = Vec::new();
    for j in split_jobs(spec) {
        let parts: Vec<&str> = j.splitn(5, '|').collect();
        if parts.len() < 3 {
            continue;
        }

        let method = parts[0].trim().to_uppercase();
        let allowed = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
        if !allowed.contains(&method.as_str()) {
            continue;
        }

        let url = parts[1].trim().to_string();
        let schedule = match Schedule::from_str(parts[2].trim()) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let headers = if parts.len() >= 4 { parse_headers(parts[3]) } else { Vec::new() };
        let body = if parts.len() == 5 {
            let b = parts[4].to_string();
            if b.is_empty() { None } else { Some(b) }
        } else {
            None
        };

        if let Some(next) = schedule.upcoming(Utc).next() {
            v.push(Job { method, url, schedule, next_fire: next, headers, body });
        }
    }
    v
}

fn apply_headers<B>(
    mut req: ureq::RequestBuilder<B>,
    secret: &str,
    headers: &[(String, String)],
) -> ureq::RequestBuilder<B> {
    req = req.header("X-Cron-Secret", secret);
    let mut has_ua = false;
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("X-Cron-Secret") {
            continue;
        }
        if k.eq_ignore_ascii_case("User-Agent") {
            has_ua = true;
        }
        req = req.header(k, v);
    }
    if !has_ua {
        req = req.header("User-Agent", "cron-runner/1.0 (Rust ureq)");
    }
    req
}

fn main() {
    let secret = env_or_exit("SECRET");
    let jobs_spec = env_or_exit("CRON_JOBS");
    let mut jobs = parse_jobs(&jobs_spec);
    if jobs.is_empty() {
        eprintln!("No valid jobs parsed from CRON_JOBS");
        exit(1);
    }

    let jitter_ms = 500i64;

    loop {
        let now = Utc::now();
        let earliest = match jobs.iter().map(|j| j.next_fire).min() {
            Some(dt) => dt,
            None => {
                eprintln!("Internal error: empty schedule set");
                exit(1);
            }
        };
        let sleep_ns = (earliest - now).num_nanoseconds().unwrap_or(0);
        if sleep_ns > 0 {
            thread::sleep(Duration::from_nanos(sleep_ns as u64));
        }

        let fired_at = Utc::now();
        for j in jobs.iter_mut() {
            if (fired_at - j.next_fire).num_milliseconds().abs() <= jitter_ms {
                let ts = Utc::now().to_rfc3339();

                let result = match j.method.as_str() {
                    "GET" => {
                        let req = apply_headers(ureq::get(&j.url), &secret, &j.headers);
                        match &j.body {
                            Some(b) => req.force_send_body().send(b.as_bytes()),
                            None => req.call(),
                        }
                    }
                    "HEAD" => {
                        let req = apply_headers(ureq::head(&j.url), &secret, &j.headers);
                        match &j.body {
                            Some(b) => req.force_send_body().send(b.as_bytes()),
                            None => req.call(),
                        }
                    }
                    "OPTIONS" => {
                        let req = apply_headers(ureq::options(&j.url), &secret, &j.headers);
                        match &j.body {
                            Some(b) => req.force_send_body().send(b.as_bytes()),
                            None => req.call(),
                        }
                    }
                    "DELETE" => {
                        let req = apply_headers(ureq::delete(&j.url), &secret, &j.headers);
                        match &j.body {
                            Some(b) => req.force_send_body().send(b.as_bytes()),
                            None => req.call(),
                        }
                    }
                    "POST" => {
                        let req = apply_headers(ureq::post(&j.url), &secret, &j.headers);
                        match &j.body {
                            Some(b) => req.send(b.as_bytes()),
                            None => req.send_empty(),
                        }
                    }
                    "PUT" => {
                        let req = apply_headers(ureq::put(&j.url), &secret, &j.headers);
                        match &j.body {
                            Some(b) => req.send(b.as_bytes()),
                            None => req.send_empty(),
                        }
                    }
                    "PATCH" => {
                        let req = apply_headers(ureq::patch(&j.url), &secret, &j.headers);
                        match &j.body {
                            Some(b) => req.send(b.as_bytes()),
                            None => req.send_empty(),
                        }
                    }
                    _ => unreachable!(),
                };

                match result {
                    Ok(resp) => println!("{} | OK | {} {} | {}", ts, j.method, j.url, resp.status()),
                    Err(ureq::Error::StatusCode(code)) => {
                        let cat = if (400..500).contains(&(code as i32)) { "client error" } else { "server error" };
                        eprintln!("{} | FAIL | {} {} | HTTP {} ({})", ts, j.method, j.url, code, cat);
                    }
                    Err(e) => eprintln!("{} | FAIL | {} {} | transport error: {}", ts, j.method, j.url, e),
                }

                if let Some(n) = j.schedule.upcoming(Utc).filter(|dt| *dt > j.next_fire).next() {
                    j.next_fire = n;
                }
            }
        }
    }
}
