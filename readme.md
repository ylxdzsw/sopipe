Sopipe
======

Socat with middlewares.

> Disclaimer:
>
> 1. Sopipe has not get security review. The encryption-related components should be used with ones own risk.
> 2. Sopipe is not designed for circumventing censorship, and shall not be used for it.
>
> The authors and contributors do not take any responsibility for abuse or misuse of this software.

## Why need it?

The 'unix way' to pipeline a stream is probably using a bach script to orchestrate the component using standard input /
output, where each component is an independent process. It is robust and flexible, however, with following problems.

1. **Protocol**: There is no standard way of passing *metadata* apart from the stream between processes. Sopipe provides
   a shared KV store with pre-defined keys for components.
2. **Performance**: Forking a process for each connection and component ranges from environmentally unfriendly to
   impractical (for 1-core VPS). Sopipe is backed by coroutine and can trivially support large numbers of short-lived
   connections.
3. **Portability**: The availability of the components varies from platform to platform, making porting the scripts a
   hassle. Sopipe is designed to be pure-rust that supports statically linking. No more cygwin. No more dockers.

## Gallery

The Gallery serves both as examples and ready-to-copy-and-paste cheatsheets for common usecases.
