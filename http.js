/* 
communication protocol: let machine to talk to each other
- http
- webRTC
- gRPC
- websockets

req-res model: client <----> server

domain name: way to reach server; google.com
ip address: domain under the hood points to ip address of server/ host machine; (google.com ---> 142.250.194.228)
- ping google.com
- dns resolution

Ports 
- logical endpoints used by protocols to identify specific processes running on computer/ server
- to run multi process on single server
- default ports
    - http > 80
    - https > 443
    - ssh > 22

methods: specify type of action that client want to perform
- get (read/ get data from server)
- post (submit data to server)
- delete (delete)
- put (update/ create)
- patch (update data)

Response
- json data (js object notation)
- html
- plaintext data

Status Code
- 200 series > success
- 300 series > redirection
- 400 series > client err
- 500 series > server err

Body
- payload w/ req

Route
- path / endpoint that define how incoming req is handled by server
- direct http req to appropriate handler fn or resources

Headers
- extra metadata of req or res
- key value pairs sent by client and server (attach bydefault)
- req headers and res headers


*/