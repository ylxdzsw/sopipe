Sopipe Component Development Guideline
======================================

0. The only entry of a component is the top-level `init` function, which returns a `&'static dyn api::Component`.

0. Components should wait for `api::Address::send` so not to overwhelm a slow component.

0. When the message queue closed (`api::Runtime::read` returns `None`), an actor should gracefully shut down itself and
   release its resources.

0. Error handling: if the error only affect a single stream, log and terminate the actor, which usually closes the
   stream. If the error is deemed fatal (e.g. some global states are corrupted), panic.
