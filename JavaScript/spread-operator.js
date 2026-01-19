let user1= {
    name: "chaitanya",
    rollno: 39,
    favColr: ["gold", "white", "grey", "silver"]
}

let user2 = {...user1}

console.log(user1.rollno)   // 39
console.log(user2.rollno)   // 39

user2.rollno= 40
console.log(user1.rollno);  // 39
console.log(user2.rollno);  // 40


user1.favColr[2]= "cream"
console.log(user1.favColr) 
console.log(user2.favColr)

/* 
spread operator is swallow copy: soln ---> serialiazattion deserialization

[ 'gold', 'white', 'cream', 'silver' ]
[ 'gold', 'white', 'cream', 'silver' ]
*/