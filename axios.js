const expreess= require('express')
const axios= require('axios')

const app= expreess()

app.get('/getdata', async (req, res) =>{
    const data= await axios.get("https://dummy-json.mock.beeceptor.com");
    console.log(data)
    console.log(data.data)
    res.json({
        msg: data.data
    })
})

app.listen(3000)


/* 
- return promise so async await
- axios auto parse into json
- fetch needs specify express.json()
- actual data is response.data


*/