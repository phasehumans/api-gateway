interface User{
    name : string,
    age : number,
    email : string
}

function WhoisOlder (user1 : User, user2 : User):string{
    if (user1.age > user2.age){
        return `${user1.name} is Older`
    }else{
        return `${user2.name} is Older`
    }
}

const ans = WhoisOlder({
    name : "chaitanyya",
    age: 20,
    email : "ereieut@gmail.com"
}, {
    name : "chetan",
    age : 21,
    email : "jhrjrh@gmail.com"
})


console.log(ans);
