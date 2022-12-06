# Multiplex SSH & HTTPs

This project is inspired by [sshl](https://github.com/yrutschle/sslh), but instead
of writing in C, I decided to write a simpler version of it using Rust. 

### What protocol can be multiplexed?
* SSH
* HTTPS

### Installation
Download the approriate binary and run directly:

```bash
# sshmultiplexer-rust --tls=192.168.1.2:443 --ssh=192.168.1.2:22 --listen=0.0.0.0:443
```

This command will listen on port 443 and redirect the traffic to `192.168.1.2:443`
if the traffic looks like a TLS traffic, redirect to `192.168.1.2:22` if ssh is detected.
Otherwise the traffic will be dropped.