# Demultiplex SSH & HTTPS

This project is inspired by [sshl](https://github.com/yrutschle/sslh), but instead
of writing in C, it's written in Rust and has very minimal functionality.

### Why is this needed?
Some public networks cut off the access to port 22 or worse, only allow some specified
ports to be accessed, like 80, 440. This app allows you to have your server serve both
HTTPS and SSH on the port 443.

### What protocol can be demultiplexed?
* SSH
* HTTPS

### Installation
Download the approriate binary and run directly:

```bash
# sshmultiplexer-rust --tls=192.168.1.2:443 --ssh=192.168.1.2:22 --listen=0.0.0.0:443
```

This command will listen on port 443 and redirect the traffic to `192.168.1.2:443`
if the traffic looks like an HTTPS traffic, redirect to `192.168.1.2:22` if ssh is detected.
Otherwise the traffic will be dropped.
