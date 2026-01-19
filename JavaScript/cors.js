/* 
cross origin resource sharing
- cross origin req are block by default
- whitelist


*/
const express= require('express')
const cors= require('cors')

const app= express()

app.use(
  cors({
    origin: "https://frontenddomain.com",
  })
);
// by default cors is block, this allows cors for that specific

app.get('/about', (req, res) => {
    res.json({
        name: "Chaitanya",
        rollno: 39,
        prn: 231107042
    })
})

app.listen(3000)
