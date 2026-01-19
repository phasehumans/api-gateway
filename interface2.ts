
interface User {
    name : string
    age : number
    email : string
}

interface Admin {
    name : string
    age : number
    access_code : number
}

// direct define after args > role : User | Admin

type UserorAdmin = User | Admin
function greet(role : UserorAdmin): string{
    return "hello" + role.name
}

const user1: User = {
    name : "chaitanya",
    age : 20,
    email: "hello@gmail.com",
}

const user2: Admin & User= {
    name : "admin10",
    age : 22,
    access_code : 394,
    email : "thisisemail@sk.com"
}


const greeting = greet(user1)           // user1.name will be called in fn > role.name
console.log(greeting)