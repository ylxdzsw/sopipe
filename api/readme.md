Sopipe Component Development Guideline
======================================

0. The only entry of a component is the top-level `init` function, which returns a `&'static dyn api::Component`.

0. Components should wait for `api::Address::send` so not to overwhelm a slow component.

0. When the message queue closed (`api::Runtime::read` returns `None`), an actor should gracefully shut down itself and
   release its resources.
