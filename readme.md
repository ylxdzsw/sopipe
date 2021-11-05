sopipe
======

Socat with middlewares.


## Why need it?

The 'unix way' to pipeline a stream is probably using a bach script to orchestrate the component using standard input / output,
where each component is an independent process. It is robust and flexible, however, with following problems.

1. **Protocol**: There is no standard way to passing *metadata* apart from the stream between processes. Sopipe provides a shared KV
   store with pre-defined keys for components.
2. **Performance**: Forking a process for each connection and component ranges from environmentally unfriendly to impractical (for
   1-core VPS). Sopipe is backed by coroutine and can trivially support large numbers of short-lived connections.
3. **Portability**: Many softwares lack the support for some platforms or behave differently on different platforms, making porting
   such scripts challenging (not to mention that even bash is not avaialiable in some OS). Sopipe is designed to be
   pure-rust that supports statically linking. No more cygwin. No more dockers.

## Gallery

The Gallery serves both as examples and ready-to-copy-and-paste cheatsheets for common usecases.
