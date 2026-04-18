# Zappaw CLI: Premium Terminal Chat Client

Zappaw CLI is a robust, high-performance Terminal User Interface (TUI) application written in Rust. It utilizes the `ratatui` ecosystem alongside `crossterm` to provide a zero-flicker, double-buffered rendering pipeline capable of 60fps animations, floating layouts, and precise terminal viewport awareness.

## Build and Execute Procedures

### System Requirements
* Cargo and Rust toolchain 1.80+

### Compiling
To compile the client locally:
```bash
cargo build --release
```

### Running the Client
To run the client interface directly from source:
```bash
cargo run
```

## Frontend Mechanics and Structure

When initiated, the application executes a visual connection boot sequence. The system relies heavily on decoupled architectural principles separating logic polling from I/O manipulation.

### Connection Wizard
Before entering the main application loop, the frontend runs a sequential data collection routine:
1. **Nickname:** Sets the public identity of the user.
2. **Room ID:** Defines the multi-cast channel the client is attempting to access.
3. **Password:** Provides a securely masked `tty` raw-mode prompt requiring secret authentication keys prior to initializing the socket backplane.

### The Rendering Pipeline (`src/ui/render.rs`)
The interface leverages `ratatui` to build exact spatial geometry dynamically across terminal boundaries.
* **Layout Constraints:** The screen utilizes a 3-tier horizontal break (Header box, Main viewport constraint, Footer input context).
* **Double Buffering:** Screen updates skip raw terminal `escape-clearing` by performing an offline differential check under the hood. Only modified ASCII points are sent via IO stream to update, preventing standard console artifact flashing during intense interactions.

## Backend Architecture Preparedness

The core data pipelines (`app.rs` and `main.rs`) have been explicitly re-architected to accommodate asynchronous remote communication vectors (such as WebSockets, gRPC, or direct TCP packet routing).

### `mpsc` Event Backplane
Synchronous interface applications historically lock on IO operations (reading keyboard streams). Zappaw overrides this via multi-threading:
* **Background Polling:** Keybind tracking and future network datagram intercepts run isolated in a standard background thread.
* **Main Loop Mutex Navigation:** The primary loop reads encapsulated structural payloads via a concurrent queue (`std::sync::mpsc::channel`), passing `AppEvent::Input(_)`, `AppEvent::Tick`, or standard custom network instructions to the frontend render thread predictably without blocking.

### Injecting Physical Backends
To attach a physical network layer to the existing frontend logic:
1. Instantiate network connections immediately following the completion of the `prompt_connection_info()` routine in `main.rs`.
2. Provide the retrieved `room` and `password` variables to your authentication socket.
3. Hook local network `Read` boundaries into the spawned `thread::spawn` background routine, and cast them as custom `AppEvent::NetworkMessage(data)` items.
4. Process received items within the master `rx.recv_timeout` handler loop in `main.rs`, mapping them into `app.messages.push()`.

## Advanced Input Mechanics

Standard interface signals have been bound into the state logic for maximum utility:
* `Ctrl+C` - Safely clears the populated Input Field buffer. If the buffer is empty, it raises a safe exit closure instruction.
* `Esc` - Clears the current prompt without submitting data.
* `Ctrl+D` - Hard shell termination hook.

## Licensing
Please reference local repository licenses for code deployment privileges.
