sopipe
======

Socat with middlewares.


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
