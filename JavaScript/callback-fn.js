// callback fn : fn which pass has args to a fn

function greet (name) {
    console.log("inside the greet fn "+ name)

}

function one (cb, name){
    console.log("inside the one fn")
    cb(name)
}


// one()       // TypeError: cb is not a function
one(greet)
// one(greet())            //TypeError: cb is not a function

one(greet, "chaitanya");




let student = {
    name: "chaitanya"
}


function grett (name){
    console.log(name)
}

let p = student.name
console.log(p)
grett(p)