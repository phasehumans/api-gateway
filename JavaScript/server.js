const http= require('http')

const server= http.createServer((req, res) => {
    console.log("incoming req ...");
    res.end("this is response")
    
})

// express creates requestHandler 
// wrapper around nodejs

server.listen(4000, () =>{
    console.log("Server started");
    

})