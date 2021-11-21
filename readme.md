Sopipe
======

Sopipe is socat with middlewares. It can be used for NAT penetration, secured* and accelerated transmission, tunnelling,
port forwarding, proxying<sup>†</sup>, etc. with arbitrarily chained encryption, compression, authentication, and error
correction.

\* Sopipe has not got security review. The encryption-related components should be used at one's own risk. <br>
<sup>†</sup> Sopipe is not designed for circumventing censorship. The authors and contributors do not take any
responsibility for abuse or misuse of this software.

## Installation

Download the latest release at [the release page](https://github.com/ylxdzsw/sopipe/releases) and drop it anywhere.
Sopipe is a single static linked binary that does not read or generate any file unless explicitly scripted.

## Usage

### Cli

Sopipe expects one and only one argument: the input script. The behaviour of sopipe is controlled solely by the script.
No commandline options are provided.

Shell tips: use single quote for the script so you don't need to escape the quotes and `!!` operations. For example:

```sh
sopipe 'stdin => exec("tee", "record.txt") !! drop => stdout'
```

If the script is long and saved in a file, you can use some shell tricks:

```sh
sopipe "$(< script.txt)"
```

Run sopipe with empty argument will print the version and enabled features.

### Script

Sopipe uses an [extreamly simple DSL](https://github.com/ylxdzsw/sopipe/blob/master/src/script.pest) to describe the
pipeline. Take a look at [the examples](https://github.com/ylxdzsw/sopipe#gallery) to get a sense of it.

A "function call" defines a node, and `=>` operators are used to connect the nodes. The arguments of a node can have
three forms: key-value pair, key-only, or value-only. If no arguments are needed, the parentheses can be omitted too.
`!!` operators can used to composite two nodes, such that the one on the left is used for forwarding and the other for
backwarding.

`:=` operator binds a node to a name. This is necessary for some nodes that expect multiple outputs. The RHS of the `:=`
operation can be a pipe `=>`, in which case the last node in the pipe is bind to the name.

`$a.b => foo()` connects `$a` and `foo()` with a specific name `b`. Some components use names to recognize the
function of each output. Named outputs can also be specified inline. For example, `foo(.b => bar()) => baz()` will connect
a `foo` component with two outputs, one is `bar()` with the name `b`, and the other is `baz()`. Anonymous outputs can
also be inlined with only a dot. For example, `stdio => tee(. => tcp(2000), . => tcp(2001))`.

## Modules

Currently the following components are avaliable. More to come™.

#### Endpoints

- [tcp]: Listen to a tcp port or send to a (remote) tcp port. If the stream is directed (e.g. produced by
  `socks5_server`), the output `tcp` node don't need arguments about destination.
- [udp]: Similar to `tcp` but for UDP.
- [stdio]: Read or write to STDIN / STDOUT.

[tcp]: https://github.com/ylxdzsw/sopipe/tree/master/components/tcp
[udp]: https://github.com/ylxdzsw/sopipe/tree/master/components/udp
[stdio]: https://github.com/ylxdzsw/sopipe/tree/master/components/stdio

#### Proxying

- [socks5]: The [SOCKS protocol](https://tools.ietf.org/html/rfc1928).

[socks5]: https://github.com/ylxdzsw/sopipe/tree/master/components/socks5

#### Authentication

- [auth]: A simple authentication components based on preshared keys and MAC. It has two methods: *time* (default) and
  *challenge*. In the *time* method, the client sends the current timestamp and MAC for verification. In the *challenge*
  method, the server actively sends a nounce and the client replies with MAC.

[auth]: https://github.com/ylxdzsw/sopipe/tree/master/components/auth

#### Encryption

- [xor]: Not really encrypt, but `xor` the stream with a fixed key.

[xor]: https://github.com/ylxdzsw/sopipe/tree/master/components/xor

#### Scripting / Debugging

- [exec]: Spawn an external process and connect to its STDIN / STDOUT. This allows integrating virtually anything with
  substantial performance penalty.
- [throttle]: Limit the flow rate like packets per second, byte per second, or randomly drop packets.
- [tee]: Broadcast to all outputs.
- [balance]: Choose one output for each stream. This has the "any" semantic while `tee` is "all".
- [drop]: Discard whatever received.
- [echo]: Reply whatever received.

[exec]: https://github.com/ylxdzsw/sopipe/tree/master/components/exec
[throttle]: https://github.com/ylxdzsw/sopipe/tree/master/components/throttle
[tee]: https://github.com/ylxdzsw/sopipe/tree/master/components/tee
[balance]: https://github.com/ylxdzsw/sopipe/tree/master/components/balance
[drop]: https://github.com/ylxdzsw/sopipe/tree/master/components/drop
[echo]: https://github.com/ylxdzsw/sopipe/tree/master/components/echo

## Performance

A micro benchmark about the local tcp port forwarding throughput using `iperf3`.

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

