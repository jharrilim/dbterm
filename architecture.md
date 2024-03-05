# Architecture

## Overview

There are three main threads of execution in the system:

- The event loop thread, which is responsible for handling terminal inputs,
  and dispatching events to the main thread.
- The render loop thread, which is responsible for writing to the terminal.
- The main thread, which is responsible for handling events and updating the
  state of the application.

