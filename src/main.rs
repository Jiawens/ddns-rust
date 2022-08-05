use curl::easy::{Easy, List};
use local_ip_address::list_afinet_netifas;
use std::io::Read;
use clap::Parser;

struct Token<'a>(&'a str, &'a str);
#[derive(Parser)]
#[clap(name = "DDNS-Rust")]
#[clap(author = "Wener <aegisa7280@gmail.com>")]
#[clap(version = "0.1.0")]
#[clap(about = "An DDNS client writtern in Rust", long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    interface_name: String,
    #[clap(short, long, value_parser)]
    sub_domain: String,
    #[clap(short, long, value_parser)]
    domain: String,
    #[clap(long, value_parser)]
    dnspod_id: String,
    #[clap(long, value_parser)]
    dnspod_token: String
}

const DNSPOD_API_DOMAIN: &str = "https://dnsapi.cn";
const USER_AGENT: &str = "UserAgent: DDNS-Rust/0.1.0 (aegisa7280@gmail.com)";

fn get_api_path(key: &str) -> Option<&str> {
    match key {
        "RecordList" => Some("/Record.List"),
        "RecordModify" => Some("/Record.Modify"),
        _ => None,
    }
}

fn log(s: &str) {
    println!("{}", s);
}

fn request_api(
    api: &str,
    token: &Token,
    parameters: Vec<(&str, &str)>,
) -> Result<json::JsonValue, String> {
    let mut post_data = format!("login_token={},{}&format=json&lang=en", token.0, token.1);
    for parament in parameters {
        let (k, v) = parament;
        post_data += format!("&{}={}", k, v).as_str();
    }
    let mut post_data = post_data.as_bytes();
    let mut result_data = Vec::new();
    let mut headers = List::new();
    headers.append(USER_AGENT).unwrap();
    let mut handle = Easy::new();
    handle
        .url(format!("{DNSPOD_API_DOMAIN}{}", get_api_path(api).unwrap()).as_str())
        .unwrap();
    handle.http_headers(headers).unwrap();
    handle.post(true).unwrap();
    handle.post_field_size(post_data.len() as u64).unwrap();
    {
        let mut transfer = handle.transfer();
        transfer
            .read_function(|buf| Ok(post_data.read(buf).unwrap_or(0)))
            .unwrap();
        transfer
            .write_function(|new_data| {
                result_data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    let result_data = String::from_utf8(result_data).unwrap();
    let result_data = json::parse(&result_data).unwrap();
    let status_code = format!("{}", result_data["status"]["code"]);
    if status_code != "1" {
        Err(status_code)
    } else {
        Ok(result_data)
    }
}

fn main() {
    let args=Args::parse();

    //Init the token to access dnspod api
    let dnspod_id = args.dnspod_id;
    let dnspod_token = args.dnspod_token;
    let token = Token(&dnspod_id, &dnspod_token);

    //Get a IPv6 address of this machine
    let mut ips: Vec<std::net::IpAddr> = Vec::new();
    {
        let network_interfaces = list_afinet_netifas().expect("Error listing network interfaces");
        let interface_name = args.interface_name;
        for (name, ip) in network_interfaces.iter() {
            if name == &interface_name && !ip.is_loopback() {
                ips.push(ip.clone());
            }
        }
        ips.retain(|x| x.is_ipv6());
        ips.reverse(); //IpAddr::is_global is still a nightly feature(github.com/rust-lang/rust/issues/27709). Reverse ips to avoid the local address in the tail. At least, for my machine.
    }
    let ip = ips
        .pop()
        .expect("Didn't find any IPv6 address on this machine");
    log(&format!("Current machine's ip: {}", ip));

    //Get the record that we want to update (Please set a record manually before first run)
    let domain = args.domain;
    let result_data = request_api("RecordList", &token, vec![("domain", &domain)])
        .expect("Request dnspod api RecordList failed with status code: ");
    let mut record_ip = json::JsonValue::String(String::from(""));
    let mut record_id = json::JsonValue::String(String::from(""));
    for record in result_data["records"].members() {
        if format!("{}",record["name"]) == format!("{}",args.sub_domain)
            && record["type"] == "AAAA"
        {
            record_ip = record["value"].clone();
            record_id = record["id"].clone();
        }
    }
    log(&format!("The record's value: {}", record_ip));

    //Check if we need to update the record
    if format!("{}", ip) != format!("{}", record_ip) {
        log("Need to update the record.");
        let record_id = format!("{}", record_id);
        request_api(
            "RecordModify",
            &token,
            vec![
                ("domain", &domain),
                ("record_id", &record_id),
                (
                    "sub_domain",
                    &args.sub_domain,
                ),
                ("record_type", "AAAA"),
                ("record_line", "默认"), //I don't know anything about this record_line, but it works fine.
                ("value", &format!("{}", ip)),
            ],
        )
        .expect("Request dnspod api RecordModify failed with status code: ");
    } else {
        log("No need to update the record.");
    }
    log("Bye bye.");
}
