
function fetch (resolve){
    setTimeout(()=>{
            return "data a gaya"
    }, 3000)
    resolve()

}

function fetchPromisified (){
    return new Promise(fetch)
}

fetchPromisified().then(()=>{
    console.log("data a gaya")
    // console.log(data)
})