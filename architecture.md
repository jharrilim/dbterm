# Architecture

## Overview

## Execution Model

There are three main threads of execution in the system:

- The event loop thread, which is responsible for handling terminal inputs,
  and dispatching events to the main thread.
- The render loop thread, which is responsible for writing to the terminal.
- The main thread, which is responsible for handling events and updating the
  state of the application.

## App Structure

The app is comprised of multiple components. Each of these components receives
a read only copy of the application state, and a channel for sending events to
the main thread to update the state. Components also handle input events in a
top-down manner. Doing event event callbacks would require dynamic dispatch,
and I figured I might as well not. Realistically that should never be a problem
anyway tbh, but I figured I'd give this a shot first.

## Rendering

There is an AppWidget trait that we are deriving instead of the standard Widget
trait. This is to allow passing in context to each render method.
