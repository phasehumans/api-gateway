const user = {
    fname : "chaitanaya",
    rollno : 39,
    address : {
        pincode : 425408,
        city :  "dondaicha"
    }
}

// const copyUser = user

const copyUser = {...user}

// console.log(user)
// console.log(copyUser)


copyUser.rollno = 99        // no change in user
copyUser.address.pincode = 696969;      // nested obj >> shallow copy >> change in user

console.log(user);
console.log(copyUser);






