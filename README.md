# DDNS-Rust

An DDNS client written in Rust. IPv6 and DNSPod only.

## Usage

### Build

`cargo build --release`

### Cli options

```
USAGE:
    ddns-rust --interface-name <INTERFACE_NAME> --sub-domain <SUB_DOMAIN> --domain <DOMAIN> --dnspod-id <DNSPOD_ID> --dnspod-token <DNSPOD_TOKEN>

OPTIONS:
    -d, --domain <DOMAIN>                    
        --dnspod-id <DNSPOD_ID>              
        --dnspod-token <DNSPOD_TOKEN>        
    -h, --help                               Print help information
    -i, --interface-name <INTERFACE_NAME>    
    -s, --sub-domain <SUB_DOMAIN>            
    -V, --version                            Print version information
```

### How to install

1. Clone this repo
2. Run `cargo build --release`
3. Found the bin ( `./target/release/ddns-rust` )
4. Move it to somewhere you like ( take `/usr/bin` for example )
5. Run `ddns-rust` , you will see something printed by ddns-rust
6. Create and edit `/usr/lib/systemd/system/ddns-rust.service` like this:

```
[Unit]
Description=An DDNS client written in Rust

[Service]
Type=oneshot
ExecStart=/PATH/TO/ddns-rust -i YOUR_INTERFACE_NAME -s YOUR_SUB_DOMAIN_NAME -d YOUR_DOMAIN_NAME --dnspod-id YOUR_DNSPOD_ID --dnspod-token YOUR_DNSPOD_TOKEN
```

Notice: The ExecStart should be like this ( If your target domain is `dr.example.com` ): `/usr/bin/ddns-rust -i wlan0 -s dr -d example.com --dnspod-id 472841 --dnspod-token 28374619272018592105e19f2a789307a`

Then create and edit '/usr/lib/systemd/system/ddns-rust.timer` like this:

```
[Unit]
Description=Run ddns-rust daily

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

7. Type `sudo systemctl start ddns-rust` in your terminal and run. There will not be any output; Have a check on DNSPod, the record should be updated ( Make sure that you had added an AAAA record )

8. Run `sudo systemctl enable ddns-rust.timer`.

9. Now, enjoy DDNS-Rust!