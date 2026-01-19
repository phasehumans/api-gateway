const express = require('express')

const app = express()

// app.method('whatroute', callback)

app.get('/', (req, res) => {
    res.send("hello world")
})

app.get('/saymyname', (req, res) => {
    res.send("Hello Chaitanya")
})


app.listen(5000)