Sopipe
======

Sopipe is socat with middlewares. It can be used for NAT penetration, secured* and accelerated transmission, tunnelling,
proxying†, etc. with arbitrarily chained encryption, compression, authentication, and error correction.

\* Sopipe has not got security review. The encryption-related components should be used at one's own risk. <br>
† Sopipe is not designed for circumventing censorship. The authors and contributors do not take any responsibility for
abuse or misuse of this software.

## Installation

Download the latest release at [the relase page](https://github.com/ylxdzsw/sopipe/releases) and drop it anywhere.
Sopipe is a single static linked binary that does not read or generate any file unless explicitly told.

## Performance

A micro benchmark about the local tcp port forwarding performance using `iperf3`.

```sh
iperf3 -s
iperf3 -c localhost -p 2000
socat -b65536 TCP-LISTEN:2000,fork TCP:127.0.0.1:5201
sopipe 'tcp(2000) => tcp("127.0.0.1", 5201)'
```

|        |  Description   |
| ------ | -------------- |
| Direct | 32.5 Gbits/sec |
| Socat  | 16.1 Gbits/sec |
| Sopipe | 12.4 Gbits/sec |

## Gallery

