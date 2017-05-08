# CC - Reverse Proxy

### Project tree

- main.rs
   - Starts a thread to handle registration requests and probe responses
   - Starts a thread to periodically send probe requests to the registered servers
- monitor.rs
   - Manages the several servers status, handling the registrations, probe requests and server picking
- server.rs
   - Deals with the operations of a single server, handling the registration requests, the sending of probe requests and determining the server's status
- packet/
   - Defines the structure of the various packets used, along with it's serialization/deserialization
- bin/monitor.rs
   - Small application that sends periodical registration requests to the proxy server and sends it it's system status when requested

**Running proxy**

`cargo run --bin reverse_proxy <IP_ADDR>`

**Running monitor**

`cargo run --bin monitor <IP_ADDR> <SERVER_IP_ADDR>`

**Note:** Both applications run on the same port so they must be used on different hosts. You can also change the port manually for testing purposes.
