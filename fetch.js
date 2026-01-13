/* 
Fetch
- use to send background req; when scroll hit req for more content from db
- send req to server in background, to get data
- need to convert fetchedd data into json etc
- axios


*/


const express= require('express')
const app= express()


app.use(express.json())

app.get('/getinfo', async (req, res) => {
    const response= await fetch("https://dummy-json.mock.beeceptor.com")
    const jsonres= response.json()

    res.json({
        result: jsonres
    })
})


app.listen(3000)