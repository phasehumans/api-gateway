
function greet(user: {
    name : string,
    age : number
}){
    console.log("hello" + user.name);
}

greet({
    name : "chaitanya",
    age : 21
})


interface UserType {
    name : string,
    age : number
}

// interfaces
function logData(user: UserType){
    console.log("name is" + user.name);
}

// types is similar to interfaces but more feat - union