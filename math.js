// named export
exports.add= function (x,y){
    return x+y
}


// default export (no multi default)
// called w/ anyname ex. userRouter
module.export= function (){
    console.log("this is default export");
    
}


module.exports= {
    add,
    sub,
}