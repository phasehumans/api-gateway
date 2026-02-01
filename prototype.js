// 

const user = {
    name: "chaitanya",
    rollno: 39,
    greet: function(){
        console.log(`hello ${this.name}`)
        // this bind context to user
    }
}

user.greet()


const user2= {
    name: "chetan"
}

// user2.greet()            ---> TypeError: user2.greet is not a function

// prototype chaining
user2.__proto__= user

user2.greet()